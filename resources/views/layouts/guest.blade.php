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

        <!-- Scripts -->
        @vite(['resources/css/app.css', 'resources/js/app.js'])
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
    </body>
</html>
