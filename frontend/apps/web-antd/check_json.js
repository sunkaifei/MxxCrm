const fs = require('fs');

function checkBraceBalance(filepath) {
  const content = fs.readFileSync(filepath, 'utf8');
  let braceCount = 0;
  let inString = false;
  let escaped = false;
  let lastErrorPos = -1;
  
  for (let i = 0; i < content.length; i++) {
    const ch = content[i];
    if (escaped) { escaped = false; continue; }
    if (ch === '\\' && inString) { escaped = true; continue; }
    if (ch === '"' && !escaped) { inString = !inString; continue; }
    if (inString) continue;
    if (ch === '{') braceCount++;
    else if (ch === '}') braceCount--;
    if (braceCount < 0) {
      console.log('Extra closing brace at position', i);
      braceCount = 0;
    }
  }
  console.log('File:', filepath);
  console.log('  Brace balance:', braceCount === 0 ? 'OK' : 'UNBALANCED (' + braceCount + ' open)');
  console.log('  Length:', content.length);
}

checkBraceBalance('src/locales/langs/zh-CN/page.json');
checkBraceBalance('src/locales/langs/en-US/page.json');