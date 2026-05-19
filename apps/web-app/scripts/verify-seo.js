import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const distHtmlPath = path.join(__dirname, '..', 'dist', 'index.html');

console.log('🔍 Initiating Automated SEO Quality Review...');

if (!fs.existsSync(distHtmlPath)) {
  console.error(`❌ Build artifact not found at: ${distHtmlPath}`);
  console.error('Please run "npm run build" before validating SEO quality.');
  process.exit(1);
}

const html = fs.readFileSync(distHtmlPath, 'utf-8');
let failed = false;

function check(ruleName, condition, failMessage) {
  if (condition) {
    console.log(`  ✔ [PASS] ${ruleName}`);
  } else {
    console.error(`  ❌ [FAIL] ${ruleName}: ${failMessage}`);
    failed = true;
  }
}

// 1. Verify Title Tags
const titleRegex = /<title[^>]*>([\s\S]*?)<\/title>/i;
const titleMatch = html.match(titleRegex);
const titleText = titleMatch ? titleMatch[1].trim() : '';
check(
  'Descriptive Title Tag',
  titleText.length > 10,
  'Missing or too short <title> element. Must be > 10 characters.'
);

// 2. Verify Meta Description
const descRegex = /<meta\s+[^>]*name=["']description["'][^>]*content=["']([^"']+)["'][^>]*>/i;
const descMatch = html.match(descRegex);
const descText = descMatch ? descMatch[1].trim() : '';
check(
  'Meta Description Present',
  descText.length > 25,
  'Missing or too short meta description tag. Must be > 25 characters.'
);

// 3. Verify Heading Structure (exactly one H1)
const h1Matches = html.match(/<h1[^>]*>[\s\S]*?<\/h1>/gi) || [];
check(
  'Single H1 Element',
  h1Matches.length === 1,
  `Found ${h1Matches.length} <h1> tags. Expected exactly 1 for clear document structure.`
);

// 4. Verify Interactive Element IDs (for browser testing and accessibility)
// Find all button, input, textarea tags
const interactiveRegex = /<(button|input|textarea|select)\b([^>]*)/gi;
let match;
let interactiveCount = 0;
let missingIdCount = 0;
const ids = new Set();
let duplicateIdCount = 0;

while ((match = interactiveRegex.exec(html)) !== null) {
  interactiveCount++;
  const tagContent = match[2];
  
  // Extract id
  const idMatch = tagContent.match(/id=["']([^"']+)["']/i);
  if (!idMatch) {
    missingIdCount++;
    console.warn(`    ⚠️ Missing ID: <${match[1]} ...> does not have an 'id' attribute.`);
  } else {
    const id = idMatch[1];
    if (ids.has(id)) {
      duplicateIdCount++;
      console.error(`    ❌ Duplicate ID: '${id}' is registered multiple times.`);
    } else {
      ids.add(id);
    }
  }
}

check(
  'Interactive Element IDs present',
  missingIdCount === 0,
  `Found ${missingIdCount} out of ${interactiveCount} interactive elements missing descriptive 'id' attributes.`
);

check(
  'Unique Element IDs',
  duplicateIdCount === 0,
  `Found ${duplicateIdCount} duplicate 'id' attributes in the HTML content.`
);

// Summary
if (failed) {
  console.error('\n❌ SEO Quality Validation FAILED. Please review the errors list above.');
  process.exit(1);
} else {
  console.log('\n🌟 SEO and Asset Compliance validation PASSED with zero warnings.');
  process.exit(0);
}
