#!/usr/bin/env python3.7

import argparse
import sys
import subprocess
import shlex
import json 

def exec(command, print_output=True):
  if not isinstance(command, list):
    command = shlex.split(command)
  print('>>> {}'.format(' '.join(command)))
  process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
  lines = []
  while True:
    next_line = process.stdout.readline().decode("utf-8") 
    if next_line == '' and process.poll() is not None:
      break
    lines.append(next_line)
    if print_output:
      sys.stdout.write(next_line)
      sys.stdout.flush()
  exit_code = process.returncode
  if (exit_code == 0):
    return lines
  else:
    raise subprocess.CalledProcessError(exit_code, command)

def main(args):
  parser = argparse.ArgumentParser(description='Optional app description')
  parser.add_argument('--server-bin-out-path', required=True)
  parser.add_argument('--server-tests-out-path', required=True)
  args = parser.parse_args()

  exec('cargo build --release', print_output=False)
  build_tests_output = exec('cargo test --release --no-run --message-format=json', print_output=False)
  test_binaries_infos = [line for line in build_tests_output if 'recipe_calculator_lib' in line]
  test_binary_path = None
  for info_str in test_binaries_infos:
    info = json.loads(info_str)
    if info['profile']['test'] == True:
      test_binary_path = info['filenames'][0]
      break

  if test_binary_path is None:
    raise ValueError('Couldn\'t find test binary in: {}'.format(build_tests_output))

  server_bin_path = 'target/release/recipe_calculator_bin'
  # 'cp' instead of shutil, because 'cp' keeps files properties (the files are executables)
  exec('cp {} {}'.format(test_binary_path, args.server_tests_out_path))
  exec('cp {} {}'.format(server_bin_path, args.server_bin_out_path))
  print('Done')

if __name__ == '__main__':
  sys.exit(main(sys.argv[1:]))
