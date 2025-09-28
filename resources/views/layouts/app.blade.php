<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="h-full bg-slate-50 dark:bg-slate-950 scheme-light dark:scheme-dark">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <meta name="csrf-token" content="{{ csrf_token() }}">

        <title>{{ config('app.name', 'Laravel') }} - {{ $title ?? 'Dashboard' }}</title>

        <link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
        <link href="https://fonts.bunny.net/css?family=figtree:400,500,600&display=swap" rel="stylesheet" />

        <!-- Scripts -->
        @vite(['resources/css/app.css', 'resources/js/app.js'])
        <template data-turbo-permanent>
            @livewireStyles
        </template>
    </head>
    <body class="font-sans antialiased h-full">
        <x-partials.svg />
        <div class="min-h-full">
            <x-partials.navigation />

            <!-- Page Content -->
            <main class="">
                {{ $slot }}
            </main>
        </div>
        <template data-turbo-permanent>
            @livewireScripts
        </template>
        <script>
            document.addEventListener('turbo:before-render', (event) => {
                // Si el navegador no soporta la API, continuamos de forma normal
                if (!document.startViewTransition) {
                    return;
                }

                event.preventDefault();

                // Envolvemos el cambio de DOM en la transiciÃ³n
                document.startViewTransition(() => {
                    event.detail.resume();
                });
            });
        </script>
    </body>
</html>
