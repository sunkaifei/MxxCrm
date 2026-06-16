import { EOL } from 'node:os';
import { dateUtil, readPackageJSON } from '@vben/node-utils';
async function viteLicensePlugin(root = process.cwd()) {
  const {
    description = '',
    homepage = '',
    version = '',
  } = await readPackageJSON(root);
  return {
    apply: 'build',
    enforce: 'post',
    generateBundle: {
      handler: (_options, bundle) => {
        const date = dateUtil().format('YYYY-MM-DD ');
        const copyrightText = `/*!
  * Vben Admin
  * Version: ${version}
  * Author: vben
  * Copyright (C) 2024 Vben
  * License: MIT License
  * Description: ${description}
  * Date Created: ${date}
  * Homepage: ${homepage}
  * Contact: ann.vben@gmail.com
*/
              `.trim();
        for (const [, fileContent] of Object.entries(bundle)) {
          if (fileContent.type === 'chunk' && fileContent.isEntry) {
            const chunkContent = fileContent;
            const content = chunkContent.code;
            const updatedContent = `${copyrightText}${EOL}${content}`;
            fileContent.code = updatedContent;
          }
        }
      },
      order: 'post',
    },
    name: 'vite:license',
  };
}
export { viteLicensePlugin };
