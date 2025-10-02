import './bootstrap';
import '@tailwindplus/elements';
import { gsap } from 'gsap';
import { ScrollTrigger } from 'gsap/ScrollTrigger';

gsap.registerPlugin(ScrollTrigger);

// Theme Toggle System
function initTheme() {
    // Check for saved theme preference or default to 'system'
    const savedTheme = localStorage.getItem('theme') || 'system';
    applyTheme(savedTheme);

    // Sync checkbox state with current theme
    const checkbox = document.getElementById('theme-toggle');
    if (checkbox) {
        const isDark = document.documentElement.classList.contains('dark');
        checkbox.checked = isDark;
    }

    // Expose toggle function globally
    window.toggleTheme = function(theme) {
        localStorage.setItem('theme', theme);
        applyTheme(theme);
    };
}

function applyTheme(theme) {
    const html = document.documentElement;

    if (theme === 'dark') {
        html.classList.add('dark');
    } else if (theme === 'light') {
        html.classList.remove('dark');
    } else { // system
        // Check system preference
        if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
            html.classList.add('dark');
        } else {
            html.classList.remove('dark');
        }
    }
}

// Listen for system theme changes when in 'system' mode
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    const savedTheme = localStorage.getItem('theme') || 'system';
    if (savedTheme === 'system') {
        applyTheme('system');
    }
});

function initAnimations() {
    if (document.querySelector('.animated-gradient')) {
        gsap.to('.animated-gradient', {
            backgroundPosition: '400% 0',
            ease: 'power3.inOut',
            duration: 10,
            repeat: -1,
        });
    }

    if (document.querySelector('#background-blob-1')) {
        const blobTimeline = gsap.timeline({defaults: {
            ease: 'power1.inOut',
            repeat: -1,
            yoyo: true
        }});

        blobTimeline.to("#background-blob-1", {
            scale: 2,
            duration: 7
        }).to("#background-blob-2", {
            scale: 5,
            duration: 5
        }, "<").to("#background-blob-1", {
            x: 800,
            y: -200,
            duration: 10
        }, "<").to("#background-blob-2", {
            x: -1000,
            duration: 12
        }, "<");
    }
}

function updateStickyOffset() {
    const header = document.querySelector('header');
    if (!header) return;

    // Calculate navbar height and set CSS variable
    const updateOffset = () => {
        const navHeight = header.offsetHeight;
        const navTop = window.innerWidth >= 1024 ? 24 : 0; // lg:top-6 = 24px
        const totalOffset = navHeight + navTop + 16; // +16px for spacing
        document.documentElement.style.setProperty('--sticky-top-offset', `${totalOffset}px`);
    };

    // Initial calculation
    updateOffset();

    // Update on resize with debounce
    let resizeTimer;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(updateOffset, 150);
    });

    // Update on scroll (navbar might change height due to sticky behavior)
    ScrollTrigger.create({
        trigger: document.body,
        start: 'top top',
        end: 'bottom bottom',
        onUpdate: updateOffset
    });
}

document.addEventListener('DOMContentLoaded', () => {
    initTheme();
    initAnimations();
    updateStickyOffset();
});
