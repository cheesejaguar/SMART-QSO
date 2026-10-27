# Simulation Notebooks

- `power_energy_sim.ipynb`: an early power and energy balance at 1 s resolution over the ConOps phases, with simplified
  lighting and load models. Its inputs are illustrative, not the baseline; the current power closure is
  `docs/environment/POWER_BUDGET_ANALYSIS.md` and the mission simulation (`software/rust/sqso-sil`,
  `docs/test/SIL_MISSION_SIMULATION.md`), with values in `docs/BASELINE_PARAMETERS.md` §5.

```bash
pip install -r requirements.txt
jupyter notebook power_energy_sim.ipynb   # needs Jupyter (requirements.txt installs only the kernel)
```
