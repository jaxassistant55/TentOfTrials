 ```diff
--- a/v2/scripts/log_watchdog.pl
+++ b/v2/scripts/log_watchdog.pl
@@ -1,4 +1,4 @@
-#!/usr/bin/perl
+#!/usr/bin/env perl
 # log_watchdog.pl - Tail log files and alert on matching patterns
 #
 # Usage: log_watchdog.pl [config_file]
@@ -14,6 +14,7 @@
 use POSIX qw(:signal_h);
 use Time::HiRes qw(time sleep);
 use Getopt::Long;
+ Perl
+use JSON::PP;
 
 # Default configuration
 my $DEFAULT_CONFIG_FILE = '/etc/tentoftrials/log_watchdog.json';
@@ -21,6 +22,7 @@
 # Global state
 my $running = 1;
 my $config_file = $DEFAULT_CONFIG_FILE;
+my $config;  # Current loaded configuration
 my %alert_cooldowns;      # pattern -> last_alert_time
 my %pattern_matches;      # pattern -> count (for threshold tracking)
 my $last_config_mtime = 0;
@@ -28,6 +30,7 @@
 # Slack webhook (loaded from config)
 my $slack_webhook_url;
 my $slack_proxy_url;
+my $ua;  # LWP::UserAgent instance
 
 # Logging
 sub log_msg {
@@ -36,6 +39,7 @@
     print STDERR "[$ts] $level: $msg\n";
 }
 
+# Signal handlers
 sub handle_sigterm {
     my $sig = shift;
     log_msg("Received $sig, shutting down gracefully", "INFO");
@@ -47,7 +51,7 @@
     log_msg("Received SIGHUP, reloading configuration", "INFO");
     eval {
         load_config();
-        # TODO: Actually reload configuration
+        log_msg("Configuration reloaded successfully", "INFO");
     };
     if ($@) {
         log_msg("Configuration reload failed: $@", "ERROR");
@@ -58,6 +62,7 @@
 $SIG{TERM} = \&handle_sigterm;
 $SIG{HUP}  = \&handle_sighup;
 
+# Validate configuration structure
 sub validate_config {
     my ($cfg) = @_;
     
@@ -65,6 +70,7 @@
         die "Configuration must be a hash reference";
     }
     
+    # Validate alert_patterns if present
     if (exists $cfg->{alert_patterns}) {
         my $patterns = $cfg->{alert_patterns};
         die "alert_patterns must be an array" unless ref($patterns) eq 'ARRAY';
@@ -84,6 +90,7 @@
         }
     }
     
+    # Validate slack section if present
     if (exists $cfg->{slack}) {
         my $slack = $cfg->{slack};
         die "slack must be a hash" unless ref($slack) eq 'HASH';
@@ -96,6 +103,7 @@
         }
     }
     
+    # Validate global settings
     if (exists $cfg->{global_cooldown}) {
         my $gc = $cfg->{global_cooldown};
         die "global_cooldown must beSeconds" unless $gc =~ /^\d+$/;
@@ -103,6 +111,7 @@
     }
 }
 
+# Load or reload configuration from file
 sub load_config {
     my $file = shift // $config_file;
     
@@ -113,6 +122,7 @@
     
     my $new_mtime = (stat($file))[9];
     
+    # Only reload if file has changed or first load
     if ($new_mtime == $last_config_mtime && $last_config_mtime > 0) {
         return 1;
     }
@@ -120,6 +130,7 @@
     open(my $fh, '<', $file) or die "Cannot open config file $file: $!";
     local $/;
     my $content = <$fh>;
+    close($fh);
     
     my $new_config;
     eval {
@@ -133,6 +144,7 @@
     validate_config($new_config);
     
     # Apply new configuration
+    $config = $new_config;
     
     # Update Slack settings
     if (exists $new_config->{slack}) {
@@ -140,6 +152,14 @@
         $slack_webhook_url = $slack->{webhook_url} if exists $slack->{webhook_url};
         $slack_proxy_url = $slack->{proxy_url} if exists $slack->{proxy_url};
         
+        # Reinitialize UserAgent with new proxy settings
+        $ua = LWP::UserAgent->new(timeout => 30);
+        if ($slack_proxy_url) {
+            $ua->proxy(['http', 'https'], $slack_proxy_url);
+            log_msg("Slack proxy configured: $slack_proxy_url", "INFO");
+        }
+        
+        # Clear old proxy if removed
+        $ua->no_proxy() unless $slack_proxy_url;
     } else {
         $slack_webhook_url = undef;
         $slack_proxy_url = undef;
@@ -148,6 +168,7 @@
     # Reset cooldown tracking for removed patterns
     my %new_patterns;
     if (exists $new_config->{alert_patterns}) {
+        $new_patterns{$_->{pattern}} = 1 for @{$new_config->{alert_patterns}};
     }
     
     # Clean up stale cooldown entries
@@ -157,6 +178,7 @@
         }
     }
     
+    # Update file modification time
     $last_config_mtime = $new_mtime;
     
     log_msg("Configuration loaded from $file", "INFO");
@@ -164,6 +186,7 @@
     return 1;
 }
 
+# Send alert to Slack with proxy support
 sub send_slack_alert {
     my ($message, $pattern_config) = @_;
     
@@ -172,6 +195,7 @@
         return 0;
     }
     
+    # Ensure UserAgent is initialized
     unless ($ua) {
         $ua = LWP::UserAgent->new(timeout => 30);
         if ($slack_proxy_url) {
@@ -179,6 +203,7 @@
         }
     }
     
+    # Build Slack payload
     my $payload = {
         text => $message,
     };
@@ -190,6 +215,7 @@
         $payload->{username} = $pattern_config->{slack_username};
     }
     
+    # Send request
     my $response = $ua->post(
         $slack_webhook_url,
         'Content-Type' => 'application/json',
@@ -204,6 +230,7 @@
     }
 }
 
+# Check if pattern is in cooldown
 sub check_cooldown {
     my ($pattern, $cooldown_seconds) = @_;
     
@@ -216,6 +243,7 @@
     return 0;
 }
 
+# Process a single log line
 sub process_line {
     my ($line, $config) = @_;
     
@@ -224,6 +252,7 @@
     my $patterns = $config->{alert_patterns};
     my $global_cooldown = $