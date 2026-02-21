/**
 * KAMI Site — Main JavaScript
 *
 * Features:
 * - Mobile navigation toggle
 * - Scroll-based animations (Intersection Observer)
 * - Plugin catalog loading from registry index.json
 * - Copy-to-clipboard for install commands
 */

(function () {
  'use strict';

  // === Registry URL ===
  const REGISTRY_URL =
    'https://raw.githubusercontent.com/Hypijump31/kami-registry/main/index.json';

  // =============================================
  // MOBILE NAV
  // =============================================
  const navToggle = document.getElementById('nav-toggle');
  const navLinks = document.getElementById('nav-links');

  if (navToggle && navLinks) {
    navToggle.addEventListener('click', () => {
      navLinks.classList.toggle('active');
      navToggle.textContent = navLinks.classList.contains('active') ? '✕' : '☰';
    });

    // Close on link click
    navLinks.querySelectorAll('a').forEach(link => {
      link.addEventListener('click', () => {
        navLinks.classList.remove('active');
        navToggle.textContent = '☰';
      });
    });
  }

  // =============================================
  // SCROLL ANIMATIONS
  // =============================================
  const animateElements = document.querySelectorAll('.animate-in');

  if ('IntersectionObserver' in window && animateElements.length > 0) {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry, index) => {
          if (entry.isIntersecting) {
            // Stagger animation
            setTimeout(() => {
              entry.target.classList.add('visible');
            }, index * 80);
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.1, rootMargin: '0px 0px -40px 0px' }
    );

    animateElements.forEach(el => observer.observe(el));
  } else {
    // Fallback: show everything
    animateElements.forEach(el => el.classList.add('visible'));
  }

  // =============================================
  // COPY TO CLIPBOARD
  // =============================================
  const installCmd = document.getElementById('install-cmd');
  const copyFeedback = document.getElementById('copy-feedback');

  if (installCmd && copyFeedback) {
    installCmd.addEventListener('click', async () => {
      const cmd = 'curl -sSf https://kami.dev/install.sh | sh';
      try {
        await navigator.clipboard.writeText(cmd);
        copyFeedback.textContent = '✓ copied!';
        copyFeedback.style.color = 'var(--green)';
        setTimeout(() => {
          copyFeedback.textContent = '📋 copy';
          copyFeedback.style.color = '';
        }, 2000);
      } catch {
        // Fallback
        copyFeedback.textContent = '⚠ use Ctrl+C';
        setTimeout(() => {
          copyFeedback.textContent = '📋 copy';
        }, 2000);
      }
    });
  }

  // Copy on plugin install commands
  document.addEventListener('click', (e) => {
    const installEl = e.target.closest('.plugin-card__install');
    if (!installEl) return;

    const text = installEl.textContent.replace('$ ', '').trim();
    navigator.clipboard.writeText(text).then(() => {
      const original = installEl.innerHTML;
      installEl.innerHTML = '<span style="color: var(--green);">✓ copied!</span>';
      setTimeout(() => { installEl.innerHTML = original; }, 1500);
    }).catch(() => {});
  });

  // =============================================
  // PLUGIN CATALOG — Load from registry
  // =============================================
  const catalogGrid = document.getElementById('catalog-grid');

  if (catalogGrid) {
    loadPlugins();
  }

  async function loadPlugins() {
    try {
      const resp = await fetch(REGISTRY_URL);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const plugins = await resp.json();

      if (!Array.isArray(plugins) || plugins.length === 0) {
        catalogGrid.innerHTML = `
          <div class="catalog__empty">
            <p style="font-size: var(--fs-lg); margin-bottom: var(--space-md);">No plugins yet</p>
            <p>Be the first to <a href="https://github.com/Hypijump31/kami/blob/main/docs/TOOL_AUTHOR_GUIDE.md">publish a plugin</a>!</p>
          </div>
        `;
        updateCount(0);
        return;
      }

      catalogGrid.innerHTML = plugins.map(p => createPluginCard(p)).join('');
      updateCount(plugins.length);
    } catch (err) {
      catalogGrid.innerHTML = `
        <div class="catalog__empty">
          <p style="margin-bottom: var(--space-sm);">Could not load the plugin registry.</p>
          <p style="font-size: var(--fs-xs);">
            <a href="${REGISTRY_URL}">View raw index.json</a> or try
            <code style="color: var(--cyan);">kami search &lt;query&gt;</code> from the CLI.
          </p>
        </div>
      `;
      console.warn('Failed to load registry:', err);
    }
  }

  function createPluginCard(plugin) {
    const installCmd = plugin.source
      ? `kami install ${plugin.source}`
      : `kami install ${plugin.id}`;

    const signedBadge = plugin.signature
      ? '<span style="color: var(--green); font-size: var(--fs-xs); font-weight: 600;" title="Ed25519 signed">✓ signed</span>'
      : '';

    return `
      <div class="plugin-card">
        <div class="plugin-card__header">
          <span class="plugin-card__name">${escapeHtml(plugin.name)}</span>
          <div style="display: flex; align-items: center; gap: var(--space-sm);">
            ${signedBadge}
            <span class="plugin-card__version">v${escapeHtml(plugin.version)}</span>
          </div>
        </div>
        <p class="plugin-card__desc">${escapeHtml(plugin.description)}</p>
        <div class="plugin-card__install" title="Click to copy">
          <span class="prompt">$</span> ${escapeHtml(installCmd)}
        </div>
      </div>
    `;
  }

  function updateCount(count) {
    const countEl = document.getElementById('plugins-count');
    if (countEl) {
      countEl.textContent = `${count} plugin${count !== 1 ? 's' : ''} available`;
    }
  }

  function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }
})();
