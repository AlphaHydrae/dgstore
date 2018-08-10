import { pick } from 'lodash';
import { resolve as resolvePath } from 'path';
import * as yargs from 'yargs';

export function program(argv = process.argv.slice(1)) {

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

    .parse(argv);

  const targets = args._;
  if (resolvePath(targets[0]) === resolvePath(args.$0)) {
    targets.shift();
  }

  if (!targets.length) {
    throw new Error('A file, directory or glob pattern must be given as the first argument');
  }

  return {
    ...pick(args, 'fullDigest', 'write'),
    targets
  };
}
