import json
import pytest
import hyperparameter as hp


@hp.param("my_app")
def my_app(x=1, y=2):
    return {"x": x, "y": y}


def test_cli_config_load(tmp_path):
    # Create config file
    config_file = tmp_path / "config.json"
    with open(config_file, "w") as f:
        json.dump({"my_app": {"x": 10, "y": 20}}, f)

    # Simulate CLI args: load config but no overrides
    import sys

    orig_argv = sys.argv
    sys.argv = ["prog", "--config", str(config_file)]

    try:
        # Launch should pick up config
        result = hp.launch(my_app)
        assert result["x"] == 10
        assert result["y"] == 20

        # Test override precedence: CLI > Config
        sys.argv = ["prog", "--config", str(config_file), "--define", "my_app.x=99"]
        result = hp.launch(my_app)
        assert result["x"] == 99
        assert result["y"] == 20

    finally:
        sys.argv = orig_argv


def test_cli_multi_config(tmp_path):
    base_cfg = tmp_path / "base.json"
    override_cfg = tmp_path / "override.json"

    with open(base_cfg, "w") as f:
        json.dump({"my_app": {"x": 1, "y": 1}}, f)
    with open(override_cfg, "w") as f:
        json.dump({"my_app": {"y": 2}}, f)

    import sys

    orig_argv = sys.argv
    sys.argv = ["prog", "-C", str(base_cfg), str(override_cfg)]

    try:
        result = hp.launch(my_app)
        assert result["x"] == 1
        assert result["y"] == 2
    finally:
        sys.argv = orig_argv
