<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="h-full bg-slate-50 dark:bg-slate-950 scheme-light dark:scheme-dark">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <meta name="csrf-token" content="{{ csrf_token() }}">

        <title>{{ config('app.name', 'Laravel') }} - {{ $title ?? 'Dashboard' }}</title>

        <link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
        <link href="https://fonts.bunny.net/css?family=figtree:400,500,600&display=swap" rel="stylesheet" />

        {{-- Theme initialization script - MUST run before page renders to prevent flash --}}
        <script>
            (function() {
                const savedTheme = localStorage.getItem('theme') || 'system';
                const html = document.documentElement;

                if (savedTheme === 'dark') {
                    html.classList.add('dark');
                } else if (savedTheme === 'light') {
                    html.classList.remove('dark');
                } else {
                    if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
                        html.classList.add('dark');
                    } else {
                        html.classList.remove('dark');
                    }
                }
            })();
        </script>

        <script>
            window.deferLoadingAlpine = (start) => {
                window.addEventListener('livewire:init', start)
            }
        </script>


        <!-- Scripts -->
        @vite(['resources/css/app.css', 'resources/js/app.js'])
        @livewireStyles
    </head>
    <body class="font-sans antialiased h-full">
        <x-partials.icon-command />
        <x-partials.svg />
        <div class="min-h-full">
            <x-partials.navigation />

            <!-- Page Content -->
            <main class="">
                {{ $slot }}
            </main>
            <x-partials.layout.footer />
        </div>
        @stack('scripts')
        @livewireScripts

        <!-- MathJax -->
        <script>
            MathJax = {
                startup: {
                    ready: () => {
                        MathJax.startup.defaultReady();
                        document.addEventListener('livewire:load', function () {
                            MathJax.typesetPromise();
                            Livewire.hook('message.processed', (message, component) => {
                                MathJax.typesetPromise();
                            });
                        });
                    }
                }
            };
        </script>
        <script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js"></script>
    </body>
</html>
