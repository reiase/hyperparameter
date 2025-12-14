import subprocess
import sys
import os

def run_script(script_name, description, args=None):
    print(f"\n[{description}]")
    cmd = [sys.executable, script_name]
    if args:
        cmd.extend(args)
    try:
        output = subprocess.check_output(
            cmd,
            cwd="benchmark",
            text=True,
            stderr=subprocess.STDOUT
        )
        print(output.strip())
        
        # Parse time from output
        for line in output.splitlines():
            if "Time:" in line and "seconds" in line:
                return float(line.split("Time:")[1].split("seconds")[0].strip())
        return None
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed: {e}")
        print(f"Output: {e.output}")
        return None
    except Exception as e:
        print(f"❌ Error: {e}")
        return None

def run_bench():
    print("=" * 60)
    print("🚀 Benchmark Suite: Parameter Access Performance (1M iters)")
    print("=" * 60)

    results = {}

    # 1. Hydra Baseline
    # Hydra needs to be run without args as it uses internal config loading
    results["Hydra (Baseline)"] = run_script(
        "bench_hydra.py", 
        "Running Hydra (Standard Access)"
    )

    # 2. Hyperparameter: Dynamic Access (Optimized)
    # The one we optimized before: ps = param_scope(); loop { ps.x }
    results["HP: Dynamic (Optimized)"] = run_script(
        "bench_hp.py",
        "Running HP: Dynamic Access (Scope Cached)"
    )

    # 3. Hyperparameter: Dynamic Access (Global Proxy)
    # bench_hp_dynamic_global.py uses param_scope.x (global proxy access)
    # Needs -D to set value as it uses run_cli()
    results["HP: Dynamic (Global Proxy)"] = run_script(
        "bench_hp_dynamic_global.py",
        "Running HP: Dynamic Access (Global Proxy)",
        args=["-D", "model.layers._0.size=10"]
    )

    # 4. Hyperparameter: Dynamic Access (Local Context)
    # bench_hp_dynamic_local.py uses with param_scope() as ps INSIDE loop (stress test)
    results["HP: Dynamic (Local Context)"] = run_script(
        "bench_hp_dynamic_local.py",
        "Running HP: Dynamic Access (Scope Created in Loop)"
    )

    # 5. Hyperparameter: Injected (Fastest)
    # bench_hp_injected.py uses function arguments (native python speed)
    # Needs -D to set value as it uses run_cli()
    results["HP: Injected (Native Speed)"] = run_script(
        "bench_hp_injected.py",
        "Running HP: Argument Injection",
        args=["-D", "layer_size=10"]
    )

    # Summary
    print("\n" + "=" * 60)
    print(f"{'Method':<35} | {'Time (s)':<10} | {'Speedup (vs Hydra)':<15}")
    print("-" * 60)
    
    baseline = results.get("Hydra (Baseline)")
    
    # Sort results by time (fastest first)
    sorted_results = sorted(
        [(k, v) for k, v in results.items() if v is not None],
        key=lambda x: x[1]
    )
    
    for name, time_val in sorted_results:
        if baseline:
            speedup = f"{baseline / time_val:.2f}x"
        else:
            speedup = "N/A"
        
        # Highlight the fastest
        prefix = "🏆 " if time_val == sorted_results[0][1] else "   "
        
        print(f"{prefix}{name:<32} | {time_val:<10.4f} | {speedup:<15}")

    print("=" * 60)

if __name__ == "__main__":
    run_bench()
