@props(['tools' => []])

<x-partials.icon-command />
<div x-data @keydown.window.prevent.cmd.k="$refs.dialog.showModal()" @keydown.window.prevent.ctrl.k="$refs.dialog.showModal()">

<div class="mt-8 text-center">
    <button command="show-modal" commandfor="dialog" class="w-72 md:w-lg inline-flex items-center justify-between gap-x-1.5 rounded-lg bg-white/80 dark:bg-slate-800/75 px-3 py-4 text-sm font-semibold text-slate-900 dark:text-white shadow-sm ring-1 ring-inset ring-slate-200 dark:ring-slate-800 hover:bg-white/90 dark:hover:bg-slate-800/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-purple-blue-600">
        <div class="flex items-center gap-x-2">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-5 text-slate-400">
                <path stroke-linecap="round" stroke-linejoin="round" d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" />
            </svg>
            <span class="text-slate-500 dark:text-slate-400">Buscar herramientas</span>
        </div>
        <div class="ml-4 hidden items-center gap-x-1.5 sm:flex">
            <kbd class="inline-flex items-center justify-center rounded bg-slate-200/75 dark:bg-slate-700/75 p-1 font-sans text-xs font-medium text-slate-500 dark:text-slate-400">
                Ctrl
            </kbd>
            <kbd class="inline-flex items-center justify-center rounded bg-slate-200/75 dark:bg-slate-700/75 p-1 font-sans text-xs font-medium text-slate-500 dark:text-slate-400">K</kbd>
        </div>
    </button>
</div>

<el-dialog>
  <dialog x-ref="dialog" id="dialog" class="backdrop:bg-transparent">
    <el-dialog-backdrop class="fixed inset-0 bg-gray-500/25 transition-opacity data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in dark:bg-gray-900/50"></el-dialog-backdrop>
    <div tabindex="0" class="fixed inset-0 w-screen overflow-y-auto p-4 sm:p-6 md:p-20">
      <el-dialog-panel class="mx-auto block max-w-2xl transform overflow-hidden rounded-xl bg-white shadow-2xl outline-1 outline-black/5 transition-all data-closed:scale-95 data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in dark:bg-gray-900 dark:-outline-offset-1 dark:outline-white/10">
        <el-command-palette>
          <div class="grid grid-cols-1 border-gray-100 dark:border-white/10">
            <input type="text" autofocus placeholder="Buscar..." class="col-start-1 row-start-1 h-12 w-full pr-4 pl-11 text-base text-gray-900 outline-hidden placeholder:text-gray-400 sm:text-sm dark:bg-gray-900 dark:text-white dark:placeholder:text-gray-500 border-full" />
            <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="pointer-events-none col-start-1 row-start-1 ml-4 size-5 self-center text-gray-400 dark:text-gray-500"><path d="M9 3.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11ZM2 9a7 7 0 1 1 12.452 4.391l3.328 3.329a.75.75 0 1 1-1.06 1.06l-3.329-3.328A7 7 0 0 1 2 9Z" clip-rule="evenodd" fill-rule="evenodd" /></svg>
          </div>
          <el-command-list class="block max-h-80 scroll-py-2 overflow-y-auto">
            <el-defaults class="block divide-y divide-gray-100 dark:divide-white/10">
                <div class="p-2">
                <h2 class="mt-4 mb-2 px-3 text-xs font-semibold text-gray-900 dark:text-gray-200">Nuevas Herramientas</h2>
                <div class="text-sm text-gray-700 dark:text-gray-300">
                  <a href="{{ route('fourier-series') }}" class="group flex cursor-default items-center rounded-md px-3 py-2 select-none focus:outline-hidden aria-selected:bg-gray-900/5 aria-selected:text-gray-900 dark:aria-selected:bg-white/5 dark:aria-selected:text-white">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 flex-none text-gray-900/40 group-aria-selected:text-gray-900 dark:text-gray-500 dark:group-aria-selected:text-white">
                        <use href="#icon-sf" />
                    </svg>
                    <span class="ml-3 flex-auto truncate">Serie de Fourier</span>
                    <span aria-hidden="true" class="ml-3 hidden flex-none text-gray-500 group-aria-selected:inline dark:text-gray-400">Ir a...</span>
                  </a>
                </div>
                <div class="text-sm text-gray-700 dark:text-gray-300">
                  <a href="{{ route('fourier-transform') }}" class="group flex cursor-default items-center rounded-md px-3 py-2 select-none focus:outline-hidden aria-selected:bg-gray-900/5 aria-selected:text-gray-900 dark:aria-selected:bg-white/5 dark:aria-selected:text-white">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 flex-none text-gray-900/40 group-aria-selected:text-gray-900 dark:text-gray-500 dark:group-aria-selected:text-white">
                      <use href="#icon-fx" />
                    </svg>
                    <span class="ml-3 flex-auto truncate">Transformada de Fourier</span>
                    <span aria-hidden="true" class="ml-3 hidden flex-none text-gray-500 group-aria-selected:inline dark:text-gray-400">Ir a...</span>
                  </a>
                </div>
                <div class="text-sm text-gray-700 dark:text-gray-300">
                  <a href="{{ route('convolution') }}" class="group flex cursor-default items-center rounded-md px-3 py-2 select-none focus:outline-hidden aria-selected:bg-gray-900/5 aria-selected:text-gray-900 dark:aria-selected:bg-white/5 dark:aria-selected:text-white">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 flex-none text-gray-900/40 group-aria-selected:text-gray-900 dark:text-gray-500 dark:group-aria-selected:text-white">
                      <use href="#icon-conv" />
                    </svg>
                    <span class="ml-3 flex-auto truncate">Convolución</span>
                    <span aria-hidden="true" class="ml-3 hidden flex-none text-gray-500 group-aria-selected:inline dark:text-gray-400">Ir a...</span>
                  </a>
                </div>
                <h2 class="mt-4 mb-2 px-3 text-xs font-semibold text-gray-900 dark:text-gray-200">Visita nuestra documentación</h2>
                <div class="text-sm text-gray-700 dark:text-gray-300">
                  <a href="https://github.com/eddndev/achronyme" class="group flex cursor-default items-center rounded-md px-3 py-2 select-none focus:outline-hidden aria-selected:bg-gray-900/5 aria-selected:text-gray-900 dark:aria-selected:bg-white/5 dark:aria-selected:text-white">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 flex-none text-gray-900/40 group-aria-selected:text-gray-900 dark:text-gray-500 dark:group-aria-selected:text-white">
                      <use href="#icon-github" />
                    </svg>
                    <span class="ml-3 flex-auto truncate">Repositorio de Achronyme</span>
                    <span aria-hidden="true" class="ml-3 hidden flex-none text-gray-500 group-aria-selected:inline dark:text-gray-400">Ir a...</span>
                  </a>
                </div>
              </div>
            </el-defaults>
            <el-command-group>
                <div class="p-2">
                <h2 class="mt-4 mb-2 px-3 text-xs font-semibold text-gray-500 dark:text-gray-400">Herramientas</h2>
                <div class="text-sm text-gray-700 dark:text-gray-300">
                    @foreach ($tools as $tool)
                        <a href="{{ $tool['url'] }}" class="group flex cursor-default items-center rounded-md px-3 py-2 select-none focus:outline-hidden aria-selected:bg-indigo-600 aria-selected:text-white dark:aria-selected:bg-indigo-500">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="size-6 flex-none text-gray-400 group-aria-selected:text-white dark:text-gray-500">
                                <use href="#{{ $tool['icon'] }}" />
                            </svg>
                            <span class="ml-3 flex-auto truncate">{{ $tool['title'] }}</span>
                        </a>
                    @endforeach
                </div>
            </div>
            </el-command-group>
          </el-command-list>
          <el-no-results hidden class="block px-6 py-14 text-center sm:px-14">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" data-slot="icon" aria-hidden="true" class="mx-auto size-6 text-gray-400 dark:text-gray-500"><path d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z" stroke-linecap="round" stroke-linejoin="round" /></svg>
            <p class="mt-4 text-sm text-gray-900 dark:text-gray-200">No se encontraron herramientas con ese término.</p>
          </el-no-results>
        </el-command-palette>
      </el-dialog-panel>
    </div>
  </dialog>
</div>