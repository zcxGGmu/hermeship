(function () {
  const doc = document.documentElement;
  doc.classList.add('js');

  const navToggle = document.querySelector('.nav__toggle');
  const navPanel = document.querySelector('.nav__panel');
  const navScrim = document.querySelector('.nav__scrim');
  const navLinks = Array.from(document.querySelectorAll('[data-nav-link]'));
  const sections = Array.from(document.querySelectorAll('main section[id]'));
  const copyButtons = Array.from(document.querySelectorAll('[data-copy-target]'));
  const mobileNav = window.matchMedia('(max-width: 860px)');

  function setNavOpen(isOpen) {
    if (!navToggle || !navPanel || !navScrim) return;
    navToggle.classList.toggle('is-open', isOpen);
    navToggle.setAttribute('aria-expanded', String(isOpen));
    navPanel.classList.toggle('is-open', isOpen);
    navScrim.hidden = !isOpen;
    navScrim.classList.toggle('is-open', isOpen);

    if (mobileNav.matches) {
      navPanel.setAttribute('aria-hidden', String(!isOpen));
    } else {
      navPanel.removeAttribute('aria-hidden');
      navScrim.hidden = true;
      navScrim.classList.remove('is-open');
    }
  }

  function syncNavMode() {
    if (!navToggle || !navPanel || !navScrim) return;

    if (mobileNav.matches) {
      const isOpen = navPanel.classList.contains('is-open');
      navPanel.setAttribute('aria-hidden', String(!isOpen));
      navToggle.setAttribute('aria-expanded', String(isOpen));
      navScrim.hidden = !isOpen;
    } else {
      navPanel.removeAttribute('aria-hidden');
      navPanel.classList.remove('is-open');
      navToggle.classList.remove('is-open');
      navToggle.setAttribute('aria-expanded', 'false');
      navScrim.hidden = true;
      navScrim.classList.remove('is-open');
    }
  }

  if (navToggle && navPanel && navScrim) {
    syncNavMode();

    navToggle.addEventListener('click', () => {
      const isOpen = !navPanel.classList.contains('is-open');
      setNavOpen(isOpen);
    });

    navScrim.addEventListener('click', () => setNavOpen(false));

    navPanel.addEventListener('click', (event) => {
      const target = event.target;
      if (target instanceof Element && target.matches('a')) {
        setNavOpen(false);
      }
    });

    document.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') {
        setNavOpen(false);
      }
    });

    if (mobileNav.addEventListener) {
      mobileNav.addEventListener('change', syncNavMode);
    } else if (mobileNav.addListener) {
      mobileNav.addListener(syncNavMode);
    }
  }

  function setActiveLink(id) {
    navLinks.forEach((link) => {
      const active = link.getAttribute('href') === `#${id}`;
      link.classList.toggle('is-active', active);
      if (active) {
        link.setAttribute('aria-current', 'true');
      } else {
        link.removeAttribute('aria-current');
      }
    });
  }

  function setActiveFromLocation() {
    const id = window.location.hash.replace('#', '');
    if (id && document.getElementById(id)) {
      setActiveLink(id);
      return true;
    }
    return false;
  }

  function setActiveFromViewport() {
    const anchorLine = 160;
    let activeId = sections[0]?.id || '';

    for (const section of sections) {
      const rect = section.getBoundingClientRect();
      if (rect.top <= anchorLine && rect.bottom > anchorLine) {
        activeId = section.id;
        break;
      }
      if (rect.top > anchorLine) {
        activeId = section.id;
        break;
      }
    }

    if (activeId) {
      setActiveLink(activeId);
    }
  }

  if ('IntersectionObserver' in window && sections.length) {
    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];

        if (visible) {
          setActiveLink(visible.target.id);
        }
      },
      {
        rootMargin: '-32% 0px -55% 0px',
        threshold: [0.22, 0.36, 0.52, 0.68],
      },
    );

    sections.forEach((section) => observer.observe(section));
    window.addEventListener('hashchange', setActiveFromLocation);
    window.addEventListener('scroll', setActiveFromViewport, { passive: true });
    if (!setActiveFromLocation()) {
      setActiveFromViewport();
    }
  } else if (sections[0]) {
    setActiveLink(sections[0].id);
  }

  function fallbackCopy(text) {
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.setAttribute('readonly', '');
    textarea.style.position = 'fixed';
    textarea.style.top = '0';
    textarea.style.left = '-9999px';
    document.body.appendChild(textarea);
    textarea.focus();
    textarea.select();
    const copied = document.execCommand('copy');
    document.body.removeChild(textarea);
    return copied;
  }

  copyButtons.forEach((button) => {
    const targetSelector = button.getAttribute('data-copy-target');
    if (!targetSelector) return;

    button.addEventListener('click', async () => {
      const target = document.querySelector(targetSelector);
      if (!target) return;

      const text = target.innerText.trim();
      if (!text) return;

      const originalLabel = button.textContent;

      const markDone = () => {
        button.textContent = '已复制';
        window.setTimeout(() => {
          button.textContent = originalLabel || '复制命令';
        }, 1400);
      };

      try {
        let copied = false;
        if (navigator.clipboard && navigator.clipboard.writeText) {
          try {
            await navigator.clipboard.writeText(text);
            copied = true;
          } catch (error) {
            copied = fallbackCopy(text);
          }
        } else {
          copied = fallbackCopy(text);
        }
        if (!copied) throw new Error('copy failed');
        markDone();
      } catch (error) {
        button.textContent = '复制失败';
        window.setTimeout(() => {
          button.textContent = originalLabel || '复制命令';
        }, 1600);
      }
    });
  });

  setActiveFromViewport();
})();
