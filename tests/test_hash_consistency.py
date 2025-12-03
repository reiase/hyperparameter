from unittest import TestCase, skipUnless

from hyperparameter.storage import has_rust_backend, xxh64


class TestHashConsistency(TestCase):
    @skipUnless(has_rust_backend, "Rust backend required for xxh64 hashing")
    def test_hash_value_matches_rust_const(self):
        # Expected values match Rust const_xxh64 with seed=42 (see core/src/xxh.rs tests)
        self.assertEqual(xxh64("12345"), 13461425039964245335)
        self.assertEqual(
            xxh64("12345678901234567890123456789012345678901234567890"),
            5815762531248152886,
        )
        self.assertEqual(xxh64("0123456789abcdefghijklmnopqrstuvwxyz"), 5308235351123835395)
