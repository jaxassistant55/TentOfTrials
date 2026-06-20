package com.tentoftrials.compliance;

import java.nio.charset.StandardCharsets;
import java.time.LocalDate;
import java.time.ZoneOffset;
import java.util.HashMap;
import java.util.Map;

public final class ComplianceReportTest {
    public static void main(String[] args) {
        reportIncludesPeriodMetadataAndAuditRows();
        emptyPeriodStillProducesStructuredReport();
        invalidDateRangeFailsClearly();
    }

    private static void reportIncludesPeriodMetadataAndAuditRows() {
        ComplianceAuditor auditor = new ComplianceAuditor("sftp://regulator.example", "user", "password");
        Map<String, Object> data = new HashMap<>();
        data.put("transaction_amount", 25000.00);

        auditor.auditCompliance("AML", data);

        LocalDate today = LocalDate.now(ZoneOffset.UTC);
        String report = new String(auditor.generateReport(today, today), StandardCharsets.UTF_8);

        assertFalse(report.isEmpty(), "report should not be empty");
        assertContains(report, "report_type,compliance_audit", "report type metadata");
        assertContains(report, "period_from," + today, "period start metadata");
        assertContains(report, "period_to," + today, "period end metadata");
        assertContains(report, "record_count,1", "record count metadata");
        assertContains(report, "check_id,check_type,timestamp,compliance_status,violations_summary", "row header");
        assertContains(report, "AML", "check type");
        assertContains(report, "NON_COMPLIANT", "compliance status");
        assertContains(report, "Transaction exceeds AML threshold", "violations summary");
    }

    private static void emptyPeriodStillProducesStructuredReport() {
        ComplianceAuditor auditor = new ComplianceAuditor("sftp://regulator.example", "user", "password");

        String report = new String(
            auditor.generateReport(LocalDate.of(2000, 1, 1), LocalDate.of(2000, 1, 31)),
            StandardCharsets.UTF_8
        );

        assertFalse(report.isEmpty(), "empty period report should not be empty");
        assertContains(report, "record_count,0", "empty record count");
        assertContains(report, "check_id,check_type,timestamp,compliance_status,violations_summary", "row header");
    }

    private static void invalidDateRangeFailsClearly() {
        ComplianceAuditor auditor = new ComplianceAuditor("sftp://regulator.example", "user", "password");
        try {
            auditor.generateReport(LocalDate.of(2026, 2, 1), LocalDate.of(2026, 1, 1));
        } catch (IllegalArgumentException error) {
            assertContains(error.getMessage(), "end date must be on or after start date", "range error message");
            return;
        }
        throw new AssertionError("Expected invalid report range to throw");
    }

    private static void assertContains(String value, String expected, String message) {
        if (!value.contains(expected)) {
            throw new AssertionError(message + ": expected to contain <" + expected + "> but was <" + value + ">");
        }
    }

    private static void assertFalse(boolean condition, String message) {
        if (condition) {
            throw new AssertionError(message);
        }
    }
}
