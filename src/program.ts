import { pick } from 'lodash';
import * as yargs from 'yargs';

export function program(argv = process.argv) {

  const args = yargs

    .option('full-digest', {
      alias: 'l',
      description: 'Show full digests instead of short prefixes',
      type: 'boolean'
    })

    .option('write', {
      alias: 'w',
      default: true,
      description: 'Store digests next to the files',
      type: 'boolean'
    })

    .argv;

  const targets = args._;
  if (!targets.length) {
    throw new Error('A file, directory or glob pattern must be given as the first argument');
  }

  return {
    ...pick(args, 'fullDigest', 'write'),
    targets
  };
}
