import chalk from 'chalk';

import { cli } from './cli';
import { program } from './program';

export function bin(argv = process.argv.slice(1)) {
  return Promise
    .resolve(argv)
    .then(program)
    .then(cli)
    .catch(err => console.error(chalk.red(err.stack)));
}
