from unittest import TestCase

from hyperparameter.rbackend import KVStorage


class TestRBackend(TestCase):
    def test_kvstorage_create(self):
        s = KVStorage()
        self.assertDictEqual(s.storage(), {})

    def test_kvstorage_update(self):
        s = KVStorage()
        s.update(
            {
                "a": 1,
                "b": 2.0,
                "c": "3",
                "d": True,
            }
        )
        self.assertDictEqual(
            s.storage(),
            {
                "a": 1,
                "b": 2.0,
                "c": "3",
                "d": True,
            },
        )

    def test_kvstorage_update_nested(self):
        s = KVStorage()
        s.update({"a": 1, "b": {"c": {"d": "2.0"}}})
        self.assertDictEqual(s.storage(), {"a": 1, "b.c.d": "2.0"})

    def test_kvstorage_put(self):
        s = KVStorage()
        s.put("a", 1)
        s.put("b", 2.0)
        s.put("c", "3")
        s.put("d", True)
        self.assertDictEqual(
            s.storage(),
            {
                "a": 1,
                "b": 2.0,
                "c": "3",
                "d": True,
            },
        )

    def test_kvstorage_get(self):
        s = KVStorage()
        s.update(
            {
                "a": 1,
                "b": 2.0,
                "c": "3",
                "d": True,
            }
        )
        self.assertEqual(s.get("a"), 1)
        self.assertEqual(s.get("b"), 2.0)
        self.assertEqual(s.get("c"), "3")
        self.assertEqual(s.get("d"), True)

    def test_kvstorage_enter_exit(self):
        s1 = KVStorage()
        s1.update(
            {
                "a": 1,
                "b": 2.0,
                "c": "3",
                "d": True,
            }
        )

        # enter s1
        s1.enter()
        s2 = KVStorage()
        self.assertEqual(s2.get("a"), 1)
        self.assertEqual(s2.get("b"), 2.0)
        self.assertEqual(s2.get("c"), "3")
        self.assertEqual(s2.get("d"), True)

        # exit s1
        s1.exit()
        s3 = KVStorage()
        try:
            self.assertEqual(s3.get("a"), None)
        except Exception as exc:
            self.assertIsInstance(exc, ValueError)

    def test_kvstorage_current(self):
        s1 = KVStorage()
        s1.update(
            {
                "a": 1,
                "b": 2.0,
                "c": "3",
                "d": True,
            }
        )

        # enter s1
        s1.enter()
        s2 = KVStorage()
        self.assertEqual(s2.get("a"), 1)
        self.assertEqual(s2.get("b"), 2.0)
        self.assertEqual(s2.get("c"), "3")
        self.assertEqual(s2.get("d"), True)

        KVStorage.current().put("a", 11)
        s2 = KVStorage()
        self.assertEqual(s1.get("a"), 11)
        self.assertEqual(s2.get("a"), 11)
        self.assertEqual(s2.get("b"), 2.0)
        self.assertEqual(s2.get("c"), "3")
        self.assertEqual(s2.get("d"), True)

        s1.exit()
