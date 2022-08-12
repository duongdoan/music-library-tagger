import os

def countFiles(dir):
  count = 0
  for root_dir, cur_dir, files in os.walk(dir):
      count += len(files)
  return count