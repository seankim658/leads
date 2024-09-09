def make_exe():
  dist = default_python_distribution(python_version = "3.10")
  policy = dist.make_python_packaging_policy()
  policy.resources_location = "in-memory"
  policy.resources_location_fallback = "filesystem-relative:prefix"

  python_config = dist.make_python_interpreter_config()

  exe = dist.to_python_executable(
      name = "leads",
      packaging_policy = policy,
      config = python_config,
  )

  exe.add_python_resources(exe.read_package_root(
      path="py",
      packages=["viz_lib"]
  ))

  exe.add_python_resources(exe.pip_install(["-r", "py/requirements.txt"]))

  return exe

register_target("exe", make_exe)

resolve_targets()
