import { pick } from 'lodash';
import * as yargs from 'yargs';

export function program(argv = process.argv) {

  const args = yargs
    .argv;

  const targets = args._;
  if (!targets.length) {
    throw new Error('A file, directory or glob pattern must be given as the first argument');
  }

  return {
    ...pick(args),
    targets
  };
}
