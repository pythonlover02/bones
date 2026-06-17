import subprocess, sys
rt, meta_file, repo, stage = sys.argv[1:]
meta = open(meta_file).read()
branch = "runtime/org.freedesktop.Platform.VulkanLayer.bones/x86_64/" + rt
r = subprocess.run(["ostree", "commit", "--repo=" + repo, "--branch=" + branch, "--subject=bones extension " + rt, "--add-metadata-string=xa.metadata=" + meta, stage])
sys.exit(r.returncode)
