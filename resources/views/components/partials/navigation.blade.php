<header class="sticky w-full z-30 top-0 lg:top-6">
  <nav aria-label="Global" class="bg-purple-blue-950/10 mx-auto flex max-w-7xl items-center justify-between p-6 lg:px-8 lg:rounded-full shadow-md dark:bg-gray-800/75 lg:backdrop-blur-xs">
    <div class="flex lg:flex-1">
      <a href="{{ route('home') }}" class="-m-1.5 p-1.5 flex items-center gap-x-3">
        <span class="sr-only">Achronyme - Engineering toolbox</span>
        <x-image.logo class="h-8 w-auto"
        src="resources/images/logo.png"
        alt="Achronyme - Engineering toolbox"
        priority=true
        />
        <p class="text-md font-bold text-gray-900 dark:text-white">Achronyme</p>
      </a>
    </div>
    <div class="flex lg:hidden">
      <button type="button" command="show-modal" commandfor="mobile-menu" class="-m-2.5 inline-flex items-center justify-center rounded-md p-2.5 text-gray-700 dark:text-gray-400">
        <span class="sr-only">Open main menu</span>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6">
          <path d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </div>
    <el-popover-group class="hidden lg:flex lg:gap-x-12">
      <a href="{{ route('fourier-series') }}" class="text-sm/6 font-semibold text-gray-900 dark:text-white">Serie de Fourier</a>
      <a href="{{ route('convolution') }}" class="text-sm/6 font-semibold text-gray-900 dark:text-white">Convolución</a>
      <div class="relative">
        
        <button popovertarget="desktop-menu-product" class="flex items-center gap-x-1 text-sm/6 font-semibold text-gray-900 dark:text-white">
          Herramientas
          <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="size-5 flex-none text-gray-400 dark:text-gray-500">
            <path d="M5.22 8.22a.75.75 0 0 1 1.06 0L10 11.94l3.72-3.72a.75.75 0 1 1 1.06 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L5.22 9.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd" fill-rule="evenodd" />
          </svg>
        </button>

        <el-popover id="desktop-menu-product" anchor="bottom" popover class="w-screen max-w-md overflow-hidden rounded-lg bg-white shadow-lg outline-1 outline-gray-900/5 transition transition-discrete [--anchor-gap:--spacing(3)] backdrop:bg-transparent open:block data-closed:translate-y-1 data-closed:opacity-0 data-enter:duration-200 data-enter:ease-out data-leave:duration-150 data-leave:ease-in dark:bg-gray-800 dark:shadow-none dark:-outline-offset-1 dark:outline-white/10">
          <div class="p-4">
            <div class="group relative flex items-center gap-x-6 rounded-lg p-4 text-sm/6 hover:bg-gray-50 dark:hover:bg-white/5">
              <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-700/50 dark:group-hover:bg-gray-700">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-indigo-600 dark:text-gray-400 dark:group-hover:text-white">
                  <use href="#icon-fx" />
                </svg>
              </div>
              <div class="flex-auto">
                <a href="{{ route('fourier-transform') }}" class="block font-semibold text-gray-900 dark:text-white">
                  Transformada de Fourier
                  <span class="absolute inset-0"></span>
                </a>
                <p class="mt-1 text-gray-600 dark:text-gray-400">Visualiza la Transformada de Fourier de una señal en el dominio de la frecuencia.</p>
              </div>
            </div>
            <div class="group relative flex items-center gap-x-6 rounded-lg p-4 text-sm/6 hover:bg-gray-50 dark:hover:bg-white/5">
              <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-700/50 dark:group-hover:bg-gray-700">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-indigo-600 dark:text-gray-400 dark:group-hover:text-white">
                  <use href="#icon-sf" />
                </svg>
              </div>
              <div class="flex-auto">
                <a href="{{ route('fourier-series') }}" class="block font-semibold text-gray-900 dark:text-white">
                  Serie de Fourier
                  <span class="absolute inset-0"></span>
                </a>
                <p class="mt-1 text-gray-600 dark:text-gray-400">Visualiza la aproximación de una señal en el dominio del tiempo.</p>
              </div>
            </div>
            <div class="group relative flex items-center gap-x-6 rounded-lg p-4 text-sm/6 hover:bg-gray-50 dark:hover:bg-white/5">
              <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-700/50 dark:group-hover:bg-gray-700">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-indigo-600 dark:text-gray-400 dark:group-hover:text-white">
                  <use href="#icon-conv" />
                </svg>
              </div>
              <div class="flex-auto">
                <a href="{{ route('convolution') }}" class="block font-semibold text-gray-900 dark:text-white">
                  Convolución
                  <span class="absolute inset-0"></span>
                </a>
                <p class="mt-1 text-gray-600 dark:text-gray-400">Visualiza la convolución de dos señales en el dominio del tiempo.</p>
              </div>
            </div>
          </div>
        </el-popover>
      </div>

      <div class="relative">
        <button popovertarget="desktop-menu-docs" class="flex items-center gap-x-1 text-sm/6 font-semibold text-gray-900 dark:text-white">
          Documentación
          <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="size-5 flex-none text-gray-400 dark:text-gray-500">
            <path d="M5.22 8.22a.75.75 0 0 1 1.06 0L10 11.94l3.72-3.72a.75.75 0 1 1 1.06 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L5.22 9.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd" fill-rule="evenodd" />
          </svg>
        </button>

        <el-popover id="desktop-menu-docs" anchor="bottom" popover class="w-screen max-w-md overflow-hidden rounded-lg bg-white shadow-lg outline-1 outline-gray-900/5 transition transition-discrete [--anchor-gap:--spacing(3)] backdrop:bg-transparent open:block data-closed:translate-y-1 data-closed:opacity-0 data-enter:duration-200 data-enter:ease-out data-leave:duration-150 data-leave:ease-in dark:bg-gray-800 dark:shadow-none dark:-outline-offset-1 dark:outline-white/10">
          <div class="p-4">
            <div class="group relative flex items-center gap-x-6 rounded-lg p-4 text-sm/6 hover:bg-gray-50 dark:hover:bg-white/5">
              <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-700/50 dark:group-hover:bg-gray-700">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-indigo-600 dark:text-gray-400 dark:group-hover:text-white">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 6.042A8.967 8.967 0 0 0 6 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 0 1 6 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 0 1 6-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0 0 18 18a8.967 8.967 0 0 0-6 2.292m0-14.25v14.25" />
                </svg>
              </div>
              <div class="flex-auto">
                <a href="#" class="block font-semibold text-gray-900 dark:text-white">
                  Documentación del Sitio
                  <span class="absolute inset-0"></span>
                </a>
                <p class="mt-1 text-gray-600 dark:text-gray-400">Aprende a usar las herramientas de procesamiento de señales.</p>
              </div>
            </div>
            <div class="group relative flex items-center gap-x-6 rounded-lg p-4 text-sm/6 hover:bg-gray-50 dark:hover:bg-white/5">
              <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-700/50 dark:group-hover:bg-gray-700">
                <svg viewBox="0 0 24 24" fill="currentColor" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-indigo-600 dark:text-gray-400 dark:group-hover:text-white">
                  <use href="#icon-github" />
                </svg>
              </div>
              <div class="flex-auto">
                <a href="https://github.com/eddndev/achronyme" target="_blank" class="block font-semibold text-gray-900 dark:text-white">
                  Repositorio GitHub
                  <span class="absolute inset-0"></span>
                </a>
                <p class="mt-1 text-gray-600 dark:text-gray-400">Revisa el código fuente y danos una ⭐ estrella.</p>
              </div>
            </div>
          </div>
        </el-popover>
      </div>
    </el-popover-group>
    <div class="hidden lg:flex lg:flex-1 lg:justify-end">
      <a href="{{ route('login') }}" class="text-sm/6 font-semibold text-gray-900 dark:text-white">Iniciar Sesión <span aria-hidden="true">&rarr;</span></a>
    </div>
  </nav>
  <el-dialog>
    <dialog id="mobile-menu" class="backdrop:bg-transparent lg:hidden">
      <div tabindex="0" class="fixed inset-0 focus:outline-none">
        <el-dialog-panel class="fixed inset-y-0 right-0 z-10 flex w-full flex-col justify-between overflow-y-auto bg-white sm:max-w-sm sm:ring-1 sm:ring-gray-900/10 dark:bg-gray-900 dark:sm:ring-white/10">
          <div class="p-6">
            <div class="flex items-center justify-between">
              <a href="{{ route('home') }}" class="-m-1.5 p-1.5 flex items-center gap-x-3">
                <span class="sr-only">Achronyme - Engineering toolbox</span>
                <x-image.logo class="h-8 w-auto"
                src="resources/images/logo.png"
                alt="Achronyme - Engineering toolbox"
                priority=true
                />
                <p class="text-md font-bold text-gray-900 dark:text-white">Achronyme</p>
              </a>
              <button type="button" command="close" commandfor="mobile-menu" class="-m-2.5 rounded-md p-2.5 text-gray-700 dark:text-gray-400">
                <span class="sr-only">Close menu</span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6">
                  <path d="M6 18 18 6M6 6l12 12" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              </button>
            </div>
            <div class="mt-6 flow-root">
              <div class="-my-6 divide-y divide-gray-500/10 dark:divide-white/10">
                <div class="space-y-2 py-6">
                  <a href="#" class="group -mx-3 flex items-center gap-x-6 rounded-lg p-3 text-base/7 font-semibold text-gray-900 hover:bg-gray-50 dark:text-white dark:hover:bg-white/5">
                    <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-800 dark:group-hover:bg-gray-700">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-purple-blue-600 dark:text-gray-300 dark:group-hover:text-white">
                        <use href="#icon-fx" />
                      </svg>
                    </div>
                    Transformada de Fourier
                  </a>
                  <a href="#" class="group -mx-3 flex items-center gap-x-6 rounded-lg p-3 text-base/7 font-semibold text-gray-900 hover:bg-gray-50 dark:text-white dark:hover:bg-white/5">
                    <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-800 dark:group-hover:bg-gray-700">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-purple-blue-600 dark:text-gray-300 dark:group-hover:text-white">
                        <use href="#icon-sf" />
                      </svg>
                    </div>
                    Serie de Fourier
                  </a>
                  <a href="#" class="group -mx-3 flex items-center gap-x-6 rounded-lg p-3 text-base/7 font-semibold text-gray-900 hover:bg-gray-50 dark:text-white dark:hover:bg-white/5">
                    <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-800 dark:group-hover:bg-gray-700">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-purple-blue-600 dark:text-gray-300 dark:group-hover:text-white">
                        <use href="#icon-conv" />
                      </svg>
                    </div>
                    Convolución
                  </a>
                </div>
                <div class="space-y-2 py-6">
                  <a href="#" class="group -mx-3 flex items-center gap-x-6 rounded-lg p-3 text-base/7 font-semibold text-gray-900 hover:bg-gray-50 dark:text-white dark:hover:bg-white/5">
                    <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-800 dark:group-hover:bg-gray-700">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-purple-blue-600 dark:text-gray-300 dark:group-hover:text-white">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 6.042A8.967 8.967 0 0 0 6 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 0 1 6 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 0 1 6-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0 0 18 18a8.967 8.967 0 0 0-6 2.292m0-14.25v14.25" />
                      </svg>
                    </div>
                    Documentación del Sitio
                  </a>
                  <a href="https://github.com/eddndev/achronyme" target="_blank" class="group -mx-3 flex items-center gap-x-6 rounded-lg p-3 text-base/7 font-semibold text-gray-900 hover:bg-gray-50 dark:text-white dark:hover:bg-white/5">
                    <div class="flex size-11 flex-none items-center justify-center rounded-lg bg-gray-50 group-hover:bg-white dark:bg-gray-800 dark:group-hover:bg-gray-700">
                      <svg viewBox="0 0 24 24" fill="currentColor" data-slot="icon" aria-hidden="true" class="size-6 text-gray-600 group-hover:text-purple-blue-600 dark:text-gray-300 dark:group-hover:text-white">
                        <use href="#icon-github" />
                      </svg>
                    </div>
                    Repositorio GitHub ⭐
                  </a>
                </div>
                <div class="py-6">
                  <a href="{{ route('login') }}" class="-mx-3 block rounded-lg px-3 py-2.5 text-base/7 font-semibold text-gray-900 hover:bg-gray-50 dark:text-white dark:hover:bg-white/5">Iniciar Sesión</a>
                </div>
              </div>
            </div>
          </div>
        </el-dialog-panel>
      </div>
    </dialog>
  </el-dialog>
</header>
