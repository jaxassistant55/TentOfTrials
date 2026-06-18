package com.tentoftrials.compliance;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Map;
import java.util.logging.Logger;
import org.junit.jupiter.api.Test;

class RuleEngineTest {
    private final RuleEngine ruleEngine = new RuleEngine(Logger.getLogger("RuleEngineTest"));

    @Test
    void kycPendingStatusFailsWithOriginalSummaryShape() {
        ComplianceAuditor.ComplianceResult result = ruleEngine.evaluate(
            "KYC",
            Map.of("user_id", "u-47", "kyc_status", "pending")
        );

        assertFalse(result.isCompliant());
        assertEquals("KYC check failed: User u-47 has not completed KYC. What the fuck?", result.getSummary());
    }

    @Test
    void amlAmountAboveThresholdFails() {
        ComplianceAuditor.ComplianceResult result = ruleEngine.evaluate(
            "AML",
            Map.of("transaction_amount", 10000.01)
        );

        assertFalse(result.isCompliant());
        assertEquals("AML flagged: Transaction exceeds AML threshold of $10000.0", result.getSummary());
    }

    @Test
    void unknownCheckTypeKeepsLegacyPassBehavior() {
        ComplianceAuditor.ComplianceResult result = ruleEngine.evaluate("COMPLY_420", Map.of());

        assertTrue(result.isCompliant());
        assertTrue(result.getViolations().isEmpty());
        assertEquals("Unknown check type: assuming compliant", result.getSummary());
    }
}
