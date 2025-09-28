import './bootstrap';
import '@hotwired/turbo';
import '@tailwindplus/elements';
import { gsap } from 'gsap';
import { ScrollTrigger } from 'gsap/ScrollTrigger';

import Alpine from 'alpinejs';

window.Alpine = Alpine;

Alpine.start();

gsap.registerPlugin(ScrollTrigger);

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

document.addEventListener('turbo:load', initAnimations);
