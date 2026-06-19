 ```diff
--- a/v2/scripts/log_watchdog.pl
+++ b/v2/scripts/log_watchdog.pl
@@ -1,4 +1,4 @@
-#!/usr/bin/env perl
+#!/usr/bin/env perl -w
 # log_watchdog.pl - Monitors log files and sends alerts via Slack
 #
 # Usage: perl log_watchdog.pl [--config <path>]
@@ -12,6 +12,7 @@
 use POSIX qw(:signal_h);
 use Time::HiRes qw(sleep);
 use Getopt::Long;
+use JSON qw(decode_json);
 
 # Default configuration
 my $DEFAULT_CONFIG_FILE = '/etc/tentoftrials/log_watchdog.conf';
@@ -19,6 +20,7 @@
 
 # Global state
 my $running = 1;
+my $reload_requested = 0;
 my %alert_cooldowns;      # pattern -> last_alert_epoch
 my %alert_counts;         # pattern -> count_today
 my $last_day_check = 0;
@@ -27,6 +29,7 @@
 my @alert_patterns;
 my $cooldown_seconds = 300;
 my $slack_webhook_url;
+my $slack_proxy_url;
 my $log_file_path;
 my $max_alerts_per_day = 100;
 my $config_file;
@@ -36,6 +39,7 @@
     my $sig_name = shift;
     log_info("Received $sig_name, reloading configuration...");
     $reload_requested = 1;
+    reload_configuration();
 }
 
 sub handle_shutdown {
@@ -44,6 +48,7 @@
     $running = 0;
 }
 
+$SIG{HUP}  = \&handle_sighup;
 $SIG{TERM} = \&handle_shutdown;
 $SIG{INT}  = \&handle_shutdown;
 
@@ -52,6 +57,7 @@
     my $config = {};
     
     if (! -f $path) {
+        log_error("Configuration file not found: $path");
         return undef;
     }
     
@@ -59,6 +65,7 @@
         my $fh;
         if (!open($fh, '<', $path)) {
             # Cannot read config
+            log_error("Cannot read configuration file: $path");
             return undef;
         }
         
@@ -66,6 +73三分快三73,6 +76,7 @@
             chomp;
             s/^\s+//; s/\s+$//;
             next if /^#/ || /^$/;
+            next if /^;/;  # Support INI-style comments too
             
             if (/^(\w+)\s*=\s*(.*)$/) {
                 my ($key, $value) = ($1, $2);
@@ -77,6 +84,7 @@
         close($fh);
     };
     if ($@) {
+        log_error("Error parsing configuration file: $@");
         return undef;
     }
     
@@ -85,6 +93,7 @@
 
 sub validate_config {
     my ($config) = @_;
+    my @errors;
     
     # Required fields
     if (!$config->{log_file}) {
@@ -95,6 +104,10 @@
         push @errors, "Missing required field: slack_webhook_url";
     }
     
+    if (@errors) {
+        return (0, join("; ", @errors));
+    }
+    
     return (1, undef);
 }
 
@@ -102,6 +115,7 @@
     my ($config) = @_;
     
     # Update alert patterns
+    my @old_patterns = @alert_patterns;
     @alert_patterns = ();
     if ($config->{alert_patterns}) {
         my @patterns = split(/\s*,\s*/, $config->{alert_patterns});
@@ -109,6 +123,7 @@
             push @alert_patterns, $pattern if length($pattern) > 0;
         }
 }
+    log_info("Alert patterns: " . (@alert_patterns ? join(", ", @alert_patterns) : "none"));
     
     # Update cooldown
     if ($config->{cooldown_seconds}) {
@@ -117,6 +132,7 @@
             $cooldown_seconds = $new_cooldown;
         }
     }
+    log_info("Cooldown seconds: $cooldown_seconds");
     
     # Update Slack webhook
     if ($config->{slack_webhook_url}) {
@@ -125,6 +141,16 @@
         }
     }
     
+    # Update Slack proxy
+    if (exists $config->{slack_proxy_url}) {
+        $slack_proxy_url = $config->{slack_proxy_url};
+        if ($slack_proxy_url) {
+            log_info("Slack proxy configured: $slack_proxy_url");
+        } else {
+            log_info("Slack proxy disabled");
+        }
+    }
+    
     # Update log file path
     if ($config->{log_file}) {
         $log_file_path = $config->{log_file};
@@ -134,6 +160,7 @@
     if ($config->{max_alerts_per_day}) {
         $max_alerts_per_day = int($config->{max_alerts_per_day});
     }
+    log_info("Max alerts per day: $max_alerts_per_day");
 }
 
 sub reload_configuration {
@@ -141,6 +168,7 @@
     
     if (! -f $config_file) {
         log_error("Configuration file not found: $config_file");
+        $reload_requested = 0;
         return 0;
     }
     
@@ -148,6 +176,7 @@
     if (!$config) {
         log_error("Failed to parse configuration file: $config_file");
         $reload_requested = 0;
         return 0;
     }
     
@@ -155,6 +184,7 @@
     if (!$valid) {
         log_error("Invalid configuration: $error");
         $reload_requested = 0;
         return 0;
     }
     
@@ -162,6 +192,7 @@
     apply_config($config);
     
     log_info("Configuration reloaded successfully from $config_file");
+    $reload_requested = 0;
     return 1;
 }
 
@@ -178,6 +209,7 @@
     my $message = shift;
     my $timestamp = strftime("%Y-%m-%d %H:%M:%S", localtime);
     print STDERR "[$timestamp] ERROR: $message\n";
+    return;
 }
 
 sub send_slack_alert {
@@ -185,6 +217,7 @@
     
     if (!$slack_webhook_url) {
         log_error("Slack webhook URL not configured");
+        return 0;
     }
     
     my $payload = {
@@ -194,6 +227,7 @@
     
     my $json_payload;
     eval {
+        require JSON;
         $json_payload = JSON::encode_json($payload);
     };
     if ($@ || !$json_payload) {
@@ -202,6 +236,7 @@
     }
     
     # Send via curl
+    my @curl_cmd = ('curl', '-s