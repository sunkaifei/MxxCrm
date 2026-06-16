import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { defineApplicationConfig } from './application';
import { defineLibraryConfig } from './library';
export * from './application';
export * from './library';
function defineConfig(userConfigPromise, type = 'auto') {
  let projectType = type;
  if (projectType === 'auto') {
    const htmlPath = join(process.cwd(), 'index.html');
    projectType = existsSync(htmlPath) ? 'application' : 'library';
  }
  switch (projectType) {
    case 'application': {
      return defineApplicationConfig(userConfigPromise);
    }
    case 'library': {
      return defineLibraryConfig(userConfigPromise);
    }
    default: {
      throw new Error(`Unsupported project type: ${projectType}`);
    }
  }
}
export { defineConfig };
