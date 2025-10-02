import { defineConfig } from 'vite';
import laravel from 'laravel-vite-plugin';
import tailwindcss from '@tailwindcss/vite';
export default defineConfig({
    plugins: [
        laravel({
            input: [
                'resources/css/app.css',
                'resources/js/app.js',
                'resources/js/fs/app.ts',
                'resources/js/ft/app.ts',
                'resources/js/convolution/app.ts',
            ],
            refresh: true,
        }),
        tailwindcss(),
    ],
    build: {
        rollupOptions: {
            output: {
                manualChunks: {
                    'chart': ['chart.js'],
                    'mathjs': ['mathjs'],
                    'gsap': ['gsap'],
                    'alpine': ['alpinejs'],
                    'elements': ['@tailwindplus/elements'],
                },
            },
        },
    },
});
