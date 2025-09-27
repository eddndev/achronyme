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
        <div class="flex min-h-full flex-col justify-center py-12 sm:px-6 lg:px-8">

            <div class="relative mt-10 sm:mx-auto sm:w-full sm:max-w-[480px]">
                {{ $slot }}
            </div>
        </div>
    </body>
</html>
