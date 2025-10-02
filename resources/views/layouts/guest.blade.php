<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="h-full bg-slate-50 dark:bg-slate-950 scheme-light dark:scheme-dark">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <meta name="csrf-token" content="{{ csrf_token() }}">

        <link rel="icon" type="image/x-icon" href="{{ asset('favicon.ico') }}">

        <title>{{ config('app.name', 'Laravel') }} - {{ $title }}</title>

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

        <!-- Scripts -->
        @vite(['resources/css/app.css', 'resources/js/app.js'])
        <template data-turbo-permanent>
            @livewireStyles
        </template>
    </head>
    <body class="font-sans text-slate-800 antialiased h-full">
        <a href="{{ route('home') }}" class="group absolute top-0 left-0 z-20 flex w-full items-center gap-x-2 p-4 transition-colors sm:p-6 lg:p-8">
            <svg viewBox="0 0 12 12" aria-hidden="true" class="size-3.5 shrink-0 stroke-slate-600 group-hover:stroke-slate-800 dark:stroke-slate-400 dark:group-hover:stroke-slate-200">
                <path d="M3.485 6.515h8.03M6 3.03l-3.03 3.03L6 9.09" fill="none" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span class="text-sm font-medium text-slate-600 group-hover:text-slate-800 dark:text-slate-400 dark:group-hover:text-slate-200">
                Volver al inicio
            </span>
        </a>
        <div class="flex min-h-full flex-col justify-center py-12 sm:px-6 lg:px-8">

            <div class="relative mt-10 sm:mx-auto sm:w-full sm:max-w-[480px]">
                {{ $slot }}
            </div>
        </div>
        <div aria-hidden="true" class="absolute inset-x-0 -top-40 -z-10 transform-gpu overflow-hidden blur-3xl sm:-top-30">
        <div id="background-blob-1" style="clip-path: polygon(74.1% 44.1%, 100% 61.6%, 97.5% 26.9%, 85.5% 0.1%, 80.7% 2%, 72.5% 32.5%, 60.2% 62.4%, 52.4% 68.1%, 47.5% 58.3%, 45.2% 34.5%, 27.5% 76.7%, 0.1% 64.9%, 17.9% 100%, 27.6% 76.8%, 76.1% 97.7%, 74.1% 44.1%)" class="relative left-[calc(50%+3rem)] aspect-1155/678 w-144.5 -translate-x-1/2 bg-linear-to-tr from-[#ff80b5] to-[#9089fc] opacity-30 sm:left-[calc(50%+36rem)] sm:w-288.75"></div>
        </div>
        <div aria-hidden="true" class="absolute inset-x-0 -top-30 -z-10 transform-gpu overflow-hidden blur-3xl sm:-top-60">
        <div id="background-blob-2" style="clip-path: polygon(74.1% 44.1%, 100% 61.6%, 97.5% 26.9%, 85.5% 0.1%, 80.7% 2%, 72.5% 32.5%, 60.2% 62.4%, 52.4% 68.1%, 47.5% 58.3%, 45.2% 34.5%, 27.5% 76.7%, 0.1% 64.9%, 17.9% 100%, 27.6% 76.8%, 76.1% 97.7%, 74.1% 44.1%)" class="relative left-[calc(50%-11rem)] aspect-1155/678 w-144.5 -translate-x-1/2 rotate-30 bg-linear-to-tr from-[#ff80b5] to-[#9089fc] opacity-30 sm:left-[calc(50%-30rem)] sm:w-288.75"></div>
        </div>
        <template data-turbo-permanent>
            @livewireScripts
        </template>
        <script>
            document.addEventListener('turbo:before-render', (event) => {
                if (!document.startViewTransition) { return; }
                event.preventDefault();
                document.startViewTransition(() => {
                    event.detail.resume();
                });
            });
        </script>
    </body>
</html>
