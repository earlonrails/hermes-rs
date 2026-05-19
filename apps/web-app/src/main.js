// Import the main CSS file so Vite compiles and bundles it
import './style.css';

// Import our build-time generated dataset
import data from './data.json';

document.addEventListener('DOMContentLoaded', () => {
  initCopyInstallWidget();
  initCommandPlayground();
  initSkillsExplorer();
  animateSveFlow();
});

/* ==========================================================================
   Click-to-copy Installation Widget
   ========================================================================== */
function initCopyInstallWidget() {
  const btn = document.getElementById('btn-copy-install');
  const commandTextEl = document.getElementById('install-command-text');
  const btnIcon = document.getElementById('copy-btn-icon');
  const btnText = document.getElementById('copy-btn-text');

  if (!btn || !commandTextEl) return;

  btn.addEventListener('click', async () => {
    const textToCopy = commandTextEl.textContent.trim();
    try {
      await navigator.clipboard.writeText(textToCopy);
      
      // Update UI feedback
      btnIcon.textContent = '✔';
      btnText.textContent = 'Copied!';
      btn.style.borderColor = 'var(--accent-cyan)';
      btn.style.color = 'var(--accent-cyan)';

      // Reset after 2 seconds
      setTimeout(() => {
        btnIcon.textContent = '📋';
        btnText.textContent = 'Copy';
        btn.style.borderColor = '';
        btn.style.color = '';
      }, 2000);
    } catch (err) {
      console.error('Failed to copy text: ', err);
    }
  });
}

/* ==========================================================================
   Interactive Terminal Playground (Typewriter Simulator)
   ========================================================================== */
function initCommandPlayground() {
  const chatTab = document.getElementById('btn-cmd-chat');
  const queryTab = document.getElementById('btn-cmd-query');
  const loginTab = document.getElementById('btn-cmd-login');
  const dashTab = document.getElementById('btn-cmd-dashboard');

  const typedInputEl = document.getElementById('terminal-typed-input');
  const outputArea = document.getElementById('terminal-output-area');
  const terminalScreen = document.getElementById('terminal-screen');

  if (!typedInputEl || !outputArea) return;

  const tabs = [
    { el: chatTab, input: 'athena chat' },
    { el: queryTab, input: 'athena query "Search for Rust files and verify tests"' },
    { el: loginTab, input: 'athena login' },
    { el: dashTab, input: 'athena dashboard' }
  ];

  let currentTypingTimeout = null;
  let activeCascades = [];

  // Find command data by input signature
  function getCommandData(inputSig) {
    return data.commands.find(c => c.input.startsWith(inputSig.split(' ')[0] + (inputSig.includes('"') || inputSig.split(' ').length > 1 ? ' ' + inputSig.split(' ')[1].replace(/"/g, '') : '')));
  }

  // Trigger typewriter simulation
  function triggerSimulation(commandInput) {
    // Clear timeouts and animations
    if (currentTypingTimeout) clearTimeout(currentTypingTimeout);
    activeCascades.forEach(t => clearTimeout(t));
    activeCascades = [];

    // Clear output
    outputArea.innerHTML = '';
    typedInputEl.textContent = '';

    // Scroll to top of terminal screen
    terminalScreen.scrollTop = 0;

    let charIndex = 0;

    function typeCharacter() {
      if (charIndex < commandInput.length) {
        typedInputEl.textContent += commandInput.charAt(charIndex);
        charIndex++;
        currentTypingTimeout = setTimeout(typeCharacter, 40);
        
        // Auto-scroll as text gets added
        terminalScreen.scrollTop = terminalScreen.scrollHeight;
      } else {
        // Typing finished -> start printing output lines
        printOutputs();
      }
    }

    function printOutputs() {
      // Find matches in dataset
      let cmdObj = data.commands.find(c => c.input === commandInput);
      
      // Fallback matching if exact not found
      if (!cmdObj) {
        cmdObj = data.commands.find(c => commandInput.startsWith(c.input.split(' ')[0]));
      }

      if (!cmdObj) return;

      cmdObj.output.forEach((line, index) => {
        const cascadeTimeout = setTimeout(() => {
          const lineEl = document.createElement('div');
          lineEl.className = 'line out-line';
          
          // Style outputs depending on status lines
          if (line.includes('✔') || line.includes('Passed')) {
            lineEl.style.color = '#34d399';
          } else if (line.includes('❌') || line.includes('Failed') || line.includes('✗')) {
            lineEl.style.color = '#f87171';
          } else if (line.includes('🤖') || line.includes('🛠️')) {
            lineEl.style.color = '#818cf8';
          } else if (line.includes('🐳')) {
            lineEl.style.color = '#38bdf8';
          } else if (line.includes('🦉') || line.includes('════')) {
            lineEl.style.color = '#a78bfa';
          } else if (line.startsWith('athena>')) {
            lineEl.style.color = '#94a3b8';
          }

          lineEl.textContent = line;
          outputArea.appendChild(lineEl);
          
          // Auto scroll terminal to the bottom
          terminalScreen.scrollTop = terminalScreen.scrollHeight;
        }, index * 250 + 150); // cascading lines delay

        activeCascades.push(cascadeTimeout);
      });
    }

    // Start typewriter
    typeCharacter();
  }

  // Bind click handlers to tabs
  tabs.forEach(tab => {
    if (!tab.el) return;
    tab.el.addEventListener('click', () => {
      // Deactivate all
      tabs.forEach(t => t.el.classList.remove('active'));
      // Activate this
      tab.el.classList.add('active');
      // Trigger typewriter
      triggerSimulation(tab.input);
    });
  });

  // Run default on page load (athena chat)
  triggerSimulation('athena chat');
}

/* ==========================================================================
   Semantic Skills Vector Memory Database Explorer
   ========================================================================== */
function initSkillsExplorer() {
  const searchInput = document.getElementById('input-skill-search');
  const listContainer = document.getElementById('skills-list-container');
  const btnClear = document.getElementById('btn-skill-clear');

  if (!listContainer) return;

  function renderSkills(skillsList) {
    listContainer.innerHTML = '';

    if (skillsList.length === 0) {
      const emptyEl = document.createElement('div');
      emptyEl.className = 'skill-db-card';
      emptyEl.style.gridColumn = '1 / -1';
      emptyEl.style.textAlign = 'center';
      emptyEl.style.padding = '40px';
      emptyEl.innerHTML = `
        <div class="feature-icon" style="font-size: 2.5rem; margin-bottom: 12px;">🔍</div>
        <h3 class="skill-card-title">No Matching Skills in Memory</h3>
        <p class="skill-section-content">The local semantic vector retriever did not find cosine similarities exceeding 0.70 thresholds.</p>
      `;
      listContainer.appendChild(emptyEl);
      return;
    }

    skillsList.forEach((skill, index) => {
      const card = document.createElement('div');
      card.className = 'skill-db-card';
      card.id = `skill-record-${index}`;

      card.innerHTML = `
        <div class="skill-card-header">
          <span class="skill-card-badge">Skill Vector #${index + 1}</span>
          <h3 class="skill-card-title">${skill.name}</h3>
        </div>
        <div>
          <div class="skill-section-title">Cosimilarity Trigger</div>
          <div class="skill-section-content">${skill.trigger}</div>
        </div>
        <div>
          <div class="skill-section-title">Agent Instructions</div>
          <div class="skill-section-content code-box">${skill.instruction}</div>
        </div>
      `;
      listContainer.appendChild(card);
    });
  }

  // Filter handler
  function performFilter() {
    const query = searchInput.value.toLowerCase().trim();
    if (!query) {
      renderSkills(data.skills);
      return;
    }

    const filtered = data.skills.filter(skill => {
      return (
        skill.name.toLowerCase().includes(query) ||
        skill.trigger.toLowerCase().includes(query) ||
        skill.instruction.toLowerCase().includes(query)
      );
    });

    renderSkills(filtered);
  }

  if (searchInput) {
    searchInput.addEventListener('input', performFilter);
  }

  if (btnClear) {
    btnClear.addEventListener('click', () => {
      if (searchInput) {
        searchInput.value = '';
      }
      renderSkills(data.skills);
    });
  }

  // Render initial list
  renderSkills(data.skills);
}

/* ==========================================================================
   Micro-animations for Crate Architecture Flowchart SVG
   ========================================================================== */
function animateSveFlow() {
  const groups = document.querySelectorAll('.arch-group');
  groups.forEach(g => {
    g.addEventListener('mouseenter', () => {
      g.style.cursor = 'pointer';
    });
  });
}
