const fs = require('fs');
['src/locales/langs/zh-CN/page.json','src/locales/langs/en-US/page.json'].forEach(f => {
  const content = fs.readFileSync(f, 'utf8').replace(/\r/g, '');
  let inStr = false, esc = false;
  let lastOpenPos = -1;
  let openCount = 0, closeCount = 0;
  // Track all { positions and their matching } positions
  let stack = [];
  
  for (let i = 0; i < content.length; i++) {
    const ch = content[i];
    if (esc) { esc = false; continue; }
    if (ch === '\\' && inStr) { esc = true; continue; }
    if (ch === '"' && !esc) { inStr = !inStr; continue; }
    if (inStr) continue;
    if (ch === '{') { stack.push(i); openCount++; }
    else if (ch === '}') { if (stack.length > 0) stack.pop(); closeCount++; }
  }
  
  console.log(f);
  console.log('  {: ' + openCount + ', }: ' + closeCount + ', unmatched: ' + stack.length);
  if (stack.length > 0) {
    // Show the last few unmatched positions
    stack.slice(-3).forEach(pos => {
      let lineNum = 1, col = 1;
      for (let j = 0; j < pos; j++) { if (content[j] === '\n') { lineNum++; col = 1; } else col++; }
      const start = Math.max(0, pos - 80);
      const end = Math.min(content.length, pos + 30);
      console.log('  Unmatched { at line ' + lineNum + ' col ' + col + ': ...' + 
        JSON.stringify(content.substring(start, end)) + '...');
    });
  }
});