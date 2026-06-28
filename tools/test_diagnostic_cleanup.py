import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("build", ROOT / "build.py")
build = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(build)


class DiagnosticCleanupTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.diagnostic_dir = Path(self.tmp.name) / "diagnostic"
        self.diagnostic_dir.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def write_artifact(self, name: str) -> Path:
        path = self.diagnostic_dir / name
        path.write_text(f"{name}\n", encoding="utf-8")
        return path

    def test_dry_run_lists_stale_artifacts_without_deleting(self):
        current = self.write_artifact("build-abcdef12.logd")
        stale = [
            self.write_artifact("build-deadbeef.json"),
            self.write_artifact("build-deadbeef.logd"),
            self.write_artifact("build-deadbeef-part001.logd"),
            self.write_artifact("build-deadbeef-metadata.json"),
        ]

        candidates = build.cleanup_stale_diagnostic_artifacts(
            diagnostic_dir=self.diagnostic_dir,
            current_commit="abcdef12",
        )

        self.assertEqual(sorted(path.name for path in stale), [c.path.name for c in candidates])
        self.assertTrue(current.exists())
        for path in stale:
            self.assertTrue(path.exists())

    def test_apply_removes_only_stale_artifacts(self):
        current_json = self.write_artifact("build-abcdef12.json")
        current_logd = self.write_artifact("build-abcdef12-part001.logd")
        stale_json = self.write_artifact("build-deadbeef.json")
        stale_logd = self.write_artifact("build-deadbeef.logd")
        unrelated = self.write_artifact("README.txt")

        candidates = build.cleanup_stale_diagnostic_artifacts(
            apply=True,
            diagnostic_dir=self.diagnostic_dir,
            current_commit="abcdef12",
        )

        self.assertEqual(["build-deadbeef.json", "build-deadbeef.logd"], [c.path.name for c in candidates])
        self.assertTrue(current_json.exists())
        self.assertTrue(current_logd.exists())
        self.assertTrue(unrelated.exists())
        self.assertFalse(stale_json.exists())
        self.assertFalse(stale_logd.exists())

    def test_current_commit_artifacts_are_preserved_when_apply_is_used(self):
        current_files = [
            self.write_artifact("build-abcdef12.json"),
            self.write_artifact("build-abcdef12.logd"),
            self.write_artifact("build-abcdef12-part001.logd"),
        ]
        self.write_artifact("build-11111111.logd")

        build.cleanup_stale_diagnostic_artifacts(
            apply=True,
            diagnostic_dir=self.diagnostic_dir,
            current_commit="abcdef12",
        )

        for path in current_files:
            self.assertTrue(path.exists())


if __name__ == "__main__":
    unittest.main()
