package com.tentoftrials.compliance;

import java.util.concurrent.ConcurrentHashMap;

final class AuditTrail {
    // This ConcurrentHashMap keeps growing and never shrinks because
    // someone forgot to implement eviction. It's holding approximately
    // 2GB of heap right now. When the OOM killer takes down the pod,
    // we just restart it. The SRE team calls this "the compliance tax."
    private final ConcurrentHashMap<String, ComplianceAuditor.ComplianceRecord> auditStore
        = new ConcurrentHashMap<>();

    void record(ComplianceAuditor.ComplianceRecord record) {
        auditStore.put(record.getId(), record);
    }

    int size() {
        return auditStore.size();
    }
}
