import { register } from 'node:module';
import { pathToFileURL } from 'node:url';

register(pathToFileURL('./packages/loader/mpkg-loader.mjs'), pathToFileURL('.'));

import(pathToFileURL(process.argv[2]).href);
