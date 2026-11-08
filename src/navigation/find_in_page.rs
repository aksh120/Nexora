// Nexora Native-Grade Find in Page Component
// Clean floating search toolbar with non-destructive highlighting, match count, and keyboard navigation.

pub struct FindInPage;

impl FindInPage {
    /// Generates the injected JavaScript that creates or focuses the floating find bar.
    pub fn script_open() -> &'static str {
        r#"(function() {
    const BAR_ID = '__nexora_find_bar__';
    let existing = document.getElementById(BAR_ID);
    if (existing) {
        existing.style.display = 'flex';
        const input = existing.querySelector('input');
        if (input) {
            input.focus();
            input.select();
        }
        return;
    }

    const bar = document.createElement('div');
    bar.id = BAR_ID;
    bar.style.cssText = `
        position: fixed;
        top: 14px;
        right: 28px;
        z-index: 2147483647;
        display: flex;
        align-items: center;
        gap: 6px;
        background: #18181B;
        border: 1px solid #27272A;
        border-radius: 8px;
        padding: 5px 8px;
        box-shadow: 0 8px 24px rgba(0,0,0,0.5), 0 2px 6px rgba(0,0,0,0.4);
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        font-size: 13px;
        color: #E4E4E7;
        user-select: none;
        backdrop-filter: blur(8px);
    `;

    // Search Icon (Lucide Search SVG)
    const iconSpan = document.createElement('span');
    iconSpan.style.cssText = 'display:flex;align-items:center;color:#71717A;margin-left:2px;';
    iconSpan.innerHTML = `<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>`;

    // Input Field
    const input = document.createElement('input');
    input.type = 'text';
    input.placeholder = 'Find in page...';
    input.style.cssText = `
        background: transparent;
        border: none;
        outline: none;
        color: #F4F4F5;
        font-size: 13px;
        font-family: inherit;
        width: 170px;
    `;

    // Match Counter
    const counter = document.createElement('span');
    counter.style.cssText = 'color:#A1A1AA;font-size:12px;min-width:32px;text-align:center;padding:0 4px;';
    counter.textContent = '0/0';

    // Button style helper
    const btnStyle = `
        background: transparent;
        border: none;
        outline: none;
        color: #A1A1AA;
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background 0.15s, color 0.15s;
    `;

    // Prev Button (ChevronUp)
    const prevBtn = document.createElement('button');
    prevBtn.style.cssText = btnStyle;
    prevBtn.title = 'Previous match (Shift+Enter)';
    prevBtn.innerHTML = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m18 15-6-6-6 6"/></svg>`;

    // Next Button (ChevronDown)
    const nextBtn = document.createElement('button');
    nextBtn.style.cssText = btnStyle;
    nextBtn.title = 'Next match (Enter)';
    nextBtn.innerHTML = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>`;

    // Close Button (X)
    const closeBtn = document.createElement('button');
    closeBtn.style.cssText = btnStyle;
    closeBtn.title = 'Close (Escape)';
    closeBtn.innerHTML = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>`;

    [prevBtn, nextBtn, closeBtn].forEach(btn => {
        btn.onmouseenter = () => { btn.style.background = '#27272A'; btn.style.color = '#F4F4F5'; };
        btn.onmouseleave = () => { btn.style.background = 'transparent'; btn.style.color = '#A1A1AA'; };
    });

    let currentIdx = 0;
    let totalMatches = 0;

    function countMatches(query) {
        if (!query) return 0;
        try {
            const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
            const regex = new RegExp(escaped, 'gi');
            const text = document.body.innerText || '';
            const matches = text.match(regex);
            return matches ? matches.length : 0;
        } catch(e) {
            return 0;
        }
    }

    function search(forward) {
        const q = input.value;
        if (!q) {
            currentIdx = 0;
            totalMatches = 0;
            counter.textContent = '0/0';
            return;
        }

        totalMatches = countMatches(q);
        const found = window.find(q, false, !forward, true, false, false, false);
        if (found) {
            if (forward) {
                currentIdx = currentIdx >= totalMatches ? 1 : currentIdx + 1;
            } else {
                currentIdx = currentIdx <= 1 ? totalMatches : currentIdx - 1;
            }
        } else {
            currentIdx = 0;
        }

        counter.textContent = totalMatches > 0 ? `${currentIdx || 1}/${totalMatches}` : '0/0';
    }

    input.addEventListener('input', () => {
        currentIdx = 0;
        if (!input.value) {
            counter.textContent = '0/0';
            try { window.getSelection().removeAllRanges(); } catch(e) {}
            return;
        }
        search(true);
    });

    input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            search(!e.shiftKey);
        } else if (e.key === 'Escape') {
            e.preventDefault();
            closeBar();
        }
    });

    prevBtn.onclick = () => search(false);
    nextBtn.onclick = () => search(true);

    function closeBar() {
        bar.style.display = 'none';
        try { window.getSelection().removeAllRanges(); } catch(e) {}
    }

    closeBtn.onclick = closeBar;

    bar.appendChild(iconSpan);
    bar.appendChild(input);
    bar.appendChild(counter);
    bar.appendChild(prevBtn);
    bar.appendChild(nextBtn);
    bar.appendChild(closeBtn);

    document.documentElement.appendChild(bar);
    input.focus();
})();"#
    }
}

