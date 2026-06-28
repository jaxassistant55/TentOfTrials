import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("build", ROOT / "build.py")
build = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(build)


class DiagnosticRetentionReportTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.diagnostic_dir = Path(self.tmp.name) / "diagnostic"
        self.diagnostic_dir.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def write_artifact(self, name: str, content: str = "artifact\n") -> Path:
        path = self.diagnostic_dir / name
        path.write_text(content, encoding="utf-8")
        return path

    def test_report_splits_current_and_older_artifacts(self):
        current_json = self.write_artifact("build-abcdef12.json", "{}")
        current_log = self.write_artifact("build-abcdef12.logd", "current-log")
        stale_log = self.write_artifact("build-deadbeef-part001.logd", "stale-log")
        stale_meta = self.write_artifact("build-deadbeef-metadata.json", "{}")
        self.write_artifact("README.txt", "not a diagnostic artifact")

        report = build.build_diagnostic_retention_report(
            diagnostic_dir=self.diagnostic_dir,
            current_commit="abcdef12",
        )

        self.assertEqual("abcdef12", report["current_commit"])
        self.assertEqual(4, report["total_artifact_count"])
        self.assertEqual(
            {"build-abcdef12.json", "build-abcdef12.logd"},
            set(report["current_artifacts"]),
        )
        self.assertEqual(
            {"build-deadbeef-part001.logd", "build-deadbeef-metadata.json"},
            set(report["older_artifacts"]),
        )
        self.assertEqual(
            sum(path.stat().st_size for path in [current_json, current_log, stale_log, stale_meta]),
            report["total_bytes"],
        )

    def test_report_is_empty_for_missing_diagnostic_dir(self):
        report = build.build_diagnostic_retention_report(
            diagnostic_dir=Path(self.tmp.name) / "missing",
            current_commit="abcdef12",
        )

        self.assertEqual(0, report["total_artifact_count"])
        self.assertEqual(0, report["total_bytes"])
        self.assertEqual([], report["current_artifacts"])
        self.assertEqual([], report["older_artifacts"])

    def test_report_entries_include_names_paths_bytes_and_current_flag(self):
        path = self.write_artifact("build-abcdef12-part001.logd", "abc")

        report = build.build_diagnostic_retention_report(
            diagnostic_dir=self.diagnostic_dir,
            current_commit="abcdef12",
        )

        self.assertEqual(
            {
                "name": "build-abcdef12-part001.logd",
                "path": str(path),
                "commit": "abcdef12",
                "bytes": 3,
                "current": True,
            },
            report["artifacts"][0],
        )


if __name__ == "__main__":
    unittest.main()
