<x-guest-layout title="Inicia sesión en tu cuenta">
    <div class="bg-white px-6 py-12 shadow-md sm:rounded-lg sm:px-12 dark:bg-slate-800 dark:border dark:border-slate-700">
    <!-- Session Status -->
        <x-auth-session-status class="mb-4" :status="session('status')" />

        <form method="POST" action="{{ route('login') }}" class="space-y-6">
            @csrf

            <!-- Email Address -->
            <div>
                <x-input-label for="email" :value="__('Email')" />
                <div class="mt-2">
                    <x-text-input id="email" class="block mt-1 w-full" type="email" name="email" :value="old('email')" required autofocus autocomplete="username" />
                </div>
                <x-input-error :messages="$errors->get('email')" class="mt-2" />
            </div>

            <!-- Password -->
            <div>
                <x-input-label for="password" :value="__('Password')" />
                <div class="mt-2">
                    <x-text-input id="password" class="block mt-1 w-full"
                                type="password"
                                name="password"
                                required autocomplete="current-password" />

                </div>
                <x-input-error :messages="$errors->get('password')" class="mt-2" />
            </div>

            <!-- Remember Me -->
            <div class="flex items-center justify-between">
                <div class="flex gap-3">
                    <div class="flex h-6 shrink-0 items-center">
                        <div class="group grid size-4 grid-cols-1">
                            <input id="remember-me" type="checkbox" name="remember-me" class="col-start-1 row-start-1 appearance-none rounded-sm border border-slate-300 bg-white checked:border-purple-blue-600 checked:bg-purple-blue-600 indeterminate:border-purple-blue-600 indeterminate:bg-purple-blue-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-blue-600 disabled:border-slate-300 disabled:bg-slate-100 disabled:checked:bg-slate-100 dark:border-slate-600 dark:bg-slate-700 dark:checked:border-purple-blue-500 dark:checked:bg-purple-blue-500 dark:indeterminate:border-purple-blue-500 dark:indeterminate:bg-purple-blue-500 dark:focus-visible:outline-purple-blue-500 forced-colors:appearance-auto" />
                            <svg viewBox="0 0 14 14" fill="none" class="pointer-events-none col-start-1 row-start-1 size-3.5 self-center justify-self-center stroke-white group-has-disabled:stroke-gray-950/25 dark:group-has-disabled:stroke-white/25">
                            <path d="M3 8L6 11L11 3.5" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-0 group-has-checked:opacity-100" />
                            <path d="M3 7H11" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-0 group-has-indeterminate:opacity-100" />
                            </svg>
                        </div>
                    </div>
                    <label for="remember-me" class="block text-sm/6 text-slate-800 dark:text-slate-200">{{ __('Remember me') }}</label>
                </div>

                <div class="text-sm/6">
                    <a href="{{ route('password.request') }}" class="font-medium text-purple-blue-600 hover:text-purple-blue-700 dark:text-purple-blue-400 dark:hover:text-purple-blue-500">¿Olvidaste tu contraseña?</a>
                </div>
            </div>

            <div class="flex items-center justify-end mt-4">

                <x-primary-button>
                    {{ __('Log in') }}
                </x-primary-button>
            </div>
        </form>

        <div>
            <div class="mt-10 flex items-center gap-x-6">
                <div class="w-full flex-1 border-t border-slate-200 dark:border-slate-700"></div>
                    <p class="text-sm/6 font-medium text-nowrap text-slate-800 dark:text-slate-200">O continuar con</p>
                <div class="w-full flex-1 border-t border-slate-200 dark:border-slate-700"></div>
            </div>

            <div class="mt-6 grid grid-cols-2 gap-4">
                <a href="{{ route('google.redirect') }}" class="flex w-full items-center justify-center gap-3 rounded-md bg-slate-200 px-3 py-2 text-sm font-semibold text-slate-700 shadow-xs border border-slate-300 hover:bg-slate-300 focus-visible:inset-ring-transparent dark:bg-slate-700 dark:text-slate-200 dark:shadow-none dark:border-slate-600 dark:hover:bg-slate-600">
                    <svg viewBox="0 0 24 24" aria-hidden="true" class="h-5 w-5">
                    <path d="M12.0003 4.75C13.7703 4.75 15.3553 5.36002 16.6053 6.54998L20.0303 3.125C17.9502 1.19 15.2353 0 12.0003 0C7.31028 0 3.25527 2.69 1.28027 6.60998L5.27028 9.70498C6.21525 6.86002 8.87028 4.75 12.0003 4.75Z" fill="#EA4335" />
                    <path d="M23.49 12.275C23.49 11.49 23.415 10.73 23.3 10H12V14.51H18.47C18.18 15.99 17.34 17.25 16.08 18.1L19.945 21.1C22.2 19.01 23.49 15.92 23.49 12.275Z" fill="#4285F4" />
                    <path d="M5.26498 14.2949C5.02498 13.5699 4.88501 12.7999 4.88501 11.9999C4.88501 11.1999 5.01998 10.4299 5.26498 9.7049L1.275 6.60986C0.46 8.22986 0 10.0599 0 11.9999C0 13.9399 0.46 15.7699 1.28 17.3899L5.26498 14.2949Z" fill="#FBBC05" />
                    <path d="M12.0004 24.0001C15.2404 24.0001 17.9654 22.935 19.9454 21.095L16.0804 18.095C15.0054 18.82 13.6204 19.245 12.0004 19.245C8.8704 19.245 6.21537 17.135 5.2654 14.29L1.27539 17.385C3.25539 21.31 7.3104 24.0001 12.0004 24.0001Z" fill="#34A853" />
                    </svg>
                    <span class="text-sm/6 font-semibold">Google</span>
                </a>

                <a href="{{ route('github.redirect') }}" class="flex w-full items-center justify-center gap-3 rounded-md bg-slate-200 px-3 py-2 text-sm font-semibold text-slate-700 shadow-xs border border-slate-300 hover:bg-slate-300 focus-visible:inset-ring-transparent dark:bg-slate-700 dark:text-slate-200 dark:shadow-none dark:border-slate-600 dark:hover:bg-slate-600">
                    <svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" class="size-5 fill-[#24292F] dark:fill-white">
                    <path d="M10 0C4.477 0 0 4.484 0 10.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0110 4.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.203 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.942.359.31.678.921.678 1.856 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0020 10.017C20 4.484 15.522 0 10 0z" clip-rule="evenodd" fill-rule="evenodd" />
                    </svg>
                    <span class="text-sm/6 font-semibold">GitHub</span>
                </a>
            </div>
        </div>
    </div>
    <p class="mt-10 text-center text-sm/6 text-slate-500 dark:text-slate-400">
    ¿No tienes una cuenta?
    <a href="{{ route('register') }}" class="font-medium text-purple-blue-600 hover:text-purple-blue-700 dark:text-purple-blue-400 dark:hover:text-purple-blue-500">Regístrate</a>
    </p>
</x-guest-layout>
