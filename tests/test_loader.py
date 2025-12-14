import json
import os
import pytest
from hyperparameter import loader


def test_deep_merge():
    base = {"a": 1, "b": {"c": 2, "d": 3}}
    override = {"b": {"c": 4}, "e": 5}

    merged = loader._merge_dicts(base, override)

    assert merged["a"] == 1
    assert merged["b"]["c"] == 4
    assert merged["b"]["d"] == 3
    assert merged["e"] == 5


def test_load_single_json(tmp_path):
    cfg_path = tmp_path / "config.json"
    data = {"a": 1, "b": "test"}
    with open(cfg_path, "w") as f:
        json.dump(data, f)

    loaded = loader.load(str(cfg_path))
    assert loaded == data


def test_load_composition(tmp_path):
    # Create base config
    base_cfg = tmp_path / "base.json"
    with open(base_cfg, "w") as f:
        json.dump({"model": {"layers": 3, "hidden": 128}, "train": {"lr": 0.01}}, f)

    # Create override config
    override_cfg = tmp_path / "override.json"
    with open(override_cfg, "w") as f:
        json.dump({"model": {"layers": 4}, "train": {"batch_size": 32}}, f)

    # Create another override (toml)
    toml_cfg = tmp_path / "final.toml"
    with open(toml_cfg, "w") as f:
        f.write("[train]\nlr = 0.001\n")

    # Test composition
    configs = loader.load([str(base_cfg), str(override_cfg), str(toml_cfg)])

    assert configs["model"]["layers"] == 4
    assert configs["model"]["hidden"] == 128  # from base
    assert configs["train"]["lr"] == 0.001  # from toml
    assert configs["train"]["batch_size"] == 32  # from override


def test_load_fallback_toml(tmp_path):
    # Test file without extension (should default to toml)
    cfg_path = tmp_path / "config"
    with open(cfg_path, "w") as f:
        f.write("a = 1\n")

    loaded = loader.load(str(cfg_path))
    assert loaded["a"] == 1
