import { Grid } from './Grid';
import { Position } from './types';

/**
 * Componente que muestra la formulación matemática del problema
 */
export class MathematicalFormulation {
  private container: HTMLElement;
  private grid: Grid;

  constructor(containerId: string, grid: Grid) {
    const container = document.getElementById(containerId);
    if (!container) {
      throw new Error(`Container with id "${containerId}" not found`);
    }
    this.container = container;
    this.grid = grid;
    this.render();
  }

  /**
   * Renderiza la formulación matemática
   */
  render(): void {
    const config = this.grid.getConfig();
    const startPos = this.grid.getStartPosition();
    const goalPositions = this.grid.getGoalPositions();
    const obstacles = this.getObstacles();

    this.container.innerHTML = `
      <div class="bg-white dark:bg-slate-800 rounded-lg shadow-lg p-6 space-y-6">
        <!-- Título -->
        <div class="border-b border-slate-200 dark:border-slate-700 pb-4">
          <h2 class="text-2xl font-bold text-slate-900 dark:text-white">Formulación Matemática del Problema</h2>
          <p class="text-sm text-slate-600 dark:text-slate-400 mt-1">Representación formal del espacio de estados</p>
        </div>

        <!-- Conjunto de Estados -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
            <span class="text-purple-blue-600 dark:text-purple-blue-400">S:</span> Conjunto de Estados
          </h3>
          <div class="bg-slate-50 dark:bg-slate-900 p-4 rounded-md border border-slate-200 dark:border-slate-700">
            <div class="font-mono text-sm">
              <span class="text-slate-700 dark:text-slate-300">S = {(a, b) | </span>
              <span class="text-purple-blue-600 dark:text-purple-blue-400">1 ≤ a ≤ ${config.rows}</span>
              <span class="text-slate-700 dark:text-slate-300"> ∧ </span>
              <span class="text-purple-blue-600 dark:text-purple-blue-400">1 ≤ b ≤ ${config.cols}</span>
              ${obstacles.length > 0 ? `
                <span class="text-slate-700 dark:text-slate-300">} \\ </span>
                <span class="text-danger">{${this.formatPositions(obstacles)}}</span>
              ` : '<span class="text-slate-700 dark:text-slate-300">}</span>'}
            </div>
            <p class="text-xs text-slate-500 dark:text-slate-400 mt-2">
              ${obstacles.length > 0
                ? `Espacio de ${config.rows * config.cols} casillas, excluyendo ${obstacles.length} obstáculo(s)`
                : `Espacio de ${config.rows * config.cols} casillas sin obstáculos`
              }
            </p>
          </div>
        </div>

        <!-- Estado Inicial -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
            <span class="text-success">s₀:</span> Estado Inicial
          </h3>
          <div class="bg-green-50 dark:bg-green-950/20 p-4 rounded-md border border-green-200 dark:border-green-800">
            <div class="font-mono text-sm">
              ${startPos
                ? `<span class="text-success dark:text-green-400">s₀ = (${startPos.row + 1}, ${startPos.col + 1})</span>`
                : `<span class="text-slate-400 dark:text-slate-500 italic">No definido</span>`
              }
            </div>
          </div>
        </div>

        <!-- Estados Finales -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
            <span class="text-danger">F:</span> Estados Finales
          </h3>
          <div class="bg-red-50 dark:bg-red-950/20 p-4 rounded-md border border-red-200 dark:border-red-800">
            <div class="font-mono text-sm">
              ${goalPositions.length > 0
                ? `<span class="text-danger dark:text-red-400">F = {${this.formatPositions(goalPositions)}}</span>`
                : `<span class="text-slate-400 dark:text-slate-500 italic">No definidos</span>`
              }
            </div>
            ${goalPositions.length > 0
              ? `<p class="text-xs text-slate-500 dark:text-slate-400 mt-2">${goalPositions.length} estado(s) objetivo</p>`
              : ''
            }
          </div>
        </div>

        <!-- Acciones -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
            <span class="text-accent-purple-600 dark:text-accent-purple-400">A:</span> Acciones Disponibles
          </h3>
          <div class="bg-purple-blue-50 dark:bg-purple-blue-950/20 p-4 rounded-md border border-purple-blue-200 dark:border-purple-blue-800 space-y-3">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <!-- Arriba -->
              <div class="flex items-center gap-2 bg-white dark:bg-slate-800 p-3 rounded border border-purple-blue-100 dark:border-purple-blue-900">
                <span class="text-2xl">⬆️</span>
                <div class="font-mono text-sm">
                  <div class="font-semibold text-purple-blue-700 dark:text-purple-blue-400">Arriba</div>
                  <div class="text-slate-600 dark:text-slate-400">(a, b) → (a-1, b)</div>
                </div>
              </div>

              <!-- Abajo -->
              <div class="flex items-center gap-2 bg-white dark:bg-slate-800 p-3 rounded border border-purple-blue-100 dark:border-purple-blue-900">
                <span class="text-2xl">⬇️</span>
                <div class="font-mono text-sm">
                  <div class="font-semibold text-purple-blue-700 dark:text-purple-blue-400">Abajo</div>
                  <div class="text-slate-600 dark:text-slate-400">(a, b) → (a+1, b)</div>
                </div>
              </div>

              <!-- Izquierda -->
              <div class="flex items-center gap-2 bg-white dark:bg-slate-800 p-3 rounded border border-purple-blue-100 dark:border-purple-blue-900">
                <span class="text-2xl">⬅️</span>
                <div class="font-mono text-sm">
                  <div class="font-semibold text-purple-blue-700 dark:text-purple-blue-400">Izquierda</div>
                  <div class="text-slate-600 dark:text-slate-400">(a, b) → (a, b-1)</div>
                </div>
              </div>

              <!-- Derecha -->
              <div class="flex items-center gap-2 bg-white dark:bg-slate-800 p-3 rounded border border-purple-blue-100 dark:border-purple-blue-900">
                <span class="text-2xl">➡️</span>
                <div class="font-mono text-sm">
                  <div class="font-semibold text-purple-blue-700 dark:text-purple-blue-400">Derecha</div>
                  <div class="text-slate-600 dark:text-slate-400">(a, b) → (a, b+1)</div>
                </div>
              </div>
            </div>
            <p class="text-xs text-slate-500 dark:text-slate-400">
              Las acciones están restringidas por los límites del tablero y los obstáculos
            </p>
          </div>
        </div>

        <!-- Función de Transición -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
            <span class="text-purple-blue-600 dark:text-purple-blue-400">δ:</span> Función de Transición
          </h3>
          <div class="bg-purple-blue-50 dark:bg-purple-blue-950/20 p-4 rounded-md border border-purple-blue-200 dark:border-purple-blue-800">
            <div class="font-mono text-sm text-slate-700 dark:text-slate-300">
              <div>δ: S × A → S</div>
              <div class="mt-2 text-xs">
                Definida cuando la acción no excede los límites ni resulta en un obstáculo
              </div>
            </div>

            <!-- Acordeón para mostrar todas las transiciones -->
            <div class="mt-4">
              <button
                id="toggle-transitions"
                class="w-full bg-white dark:bg-slate-800 px-4 py-3 rounded-md border-2 border-purple-blue-300 dark:border-purple-blue-700 hover:bg-purple-blue-100 dark:hover:bg-purple-blue-900/50 transition-colors flex items-center justify-between group"
              >
                <span class="font-semibold text-purple-blue-700 dark:text-purple-blue-400">Ver todas las transiciones definidas</span>
                <svg class="w-5 h-5 text-purple-blue-700 dark:text-purple-blue-400 transform transition-transform group-hover:scale-110" id="toggle-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                </svg>
              </button>

              <div id="transitions-content" class="hidden mt-3 bg-white dark:bg-slate-800 p-4 rounded-md border border-purple-blue-200 dark:border-purple-blue-700 max-h-96 overflow-y-auto">
                ${this.generateTransitionsTable()}
              </div>
            </div>
          </div>
        </div>
      </div>
    `;

    // Agregar event listener para el acordeón
    this.attachAccordionListener();
  }

  /**
   * Obtiene todas las posiciones de obstáculos
   */
  private getObstacles(): Position[] {
    const obstacles: Position[] = [];
    const cells = this.grid.getAllCells();

    for (let row = 0; row < cells.length; row++) {
      for (let col = 0; col < cells[row].length; col++) {
        if (cells[row][col].type === 'obstacle') {
          obstacles.push({ row, col });
        }
      }
    }

    return obstacles;
  }

  /**
   * Formatea un array de posiciones para mostrar
   */
  private formatPositions(positions: Position[]): string {
    return positions
      .map(pos => `(${pos.row + 1}, ${pos.col + 1})`)
      .join(', ');
  }

  /**
   * Genera la tabla de transiciones
   */
  private generateTransitionsTable(): string {
    const config = this.grid.getConfig();
    const obstacles = this.getObstacles();
    const obstacleSet = new Set(obstacles.map(pos => `${pos.row},${pos.col}`));

    const transitions: string[] = [];
    const actions = [
      { name: 'Arriba', icon: '⬆️', deltaRow: -1, deltaCol: 0 },
      { name: 'Abajo', icon: '⬇️', deltaRow: 1, deltaCol: 0 },
      { name: 'Izquierda', icon: '⬅️', deltaRow: 0, deltaCol: -1 },
      { name: 'Derecha', icon: '➡️', deltaRow: 0, deltaCol: 1 }
    ];

    // Generar transiciones para cada estado válido
    for (let row = 0; row < config.rows; row++) {
      for (let col = 0; col < config.cols; col++) {
        // Saltar si es un obstáculo
        if (obstacleSet.has(`${row},${col}`)) continue;

        const currentState = `(${row + 1}, ${col + 1})`;

        for (const action of actions) {
          const newRow = row + action.deltaRow;
          const newCol = col + action.deltaCol;

          // Verificar si la transición es válida
          const isValid =
            newRow >= 0 && newRow < config.rows &&
            newCol >= 0 && newCol < config.cols &&
            !obstacleSet.has(`${newRow},${newCol}`);

          if (isValid) {
            const newState = `(${newRow + 1}, ${newCol + 1})`;
            transitions.push(`
              <div class="flex items-center gap-3 py-2 px-3 hover:bg-purple-blue-50 dark:hover:bg-purple-blue-950/30 rounded transition-colors">
                <span class="text-lg">${action.icon}</span>
                <span class="font-mono text-sm text-slate-700 dark:text-slate-300">
                  δ(${currentState}, ${action.name}) = ${newState}
                </span>
              </div>
            `);
          }
        }
      }
    }

    if (transitions.length === 0) {
      return '<p class="text-slate-500 dark:text-slate-400 italic text-center py-4">No hay transiciones válidas</p>';
    }

    return `
      <div class="space-y-1">
        <p class="text-xs text-slate-600 dark:text-slate-400 mb-3">
          Total de transiciones válidas: <span class="font-semibold">${transitions.length}</span>
        </p>
        ${transitions.join('')}
      </div>
    `;
  }

  /**
   * Adjunta el listener para el acordeón
   */
  private attachAccordionListener(): void {
    const toggleBtn = document.getElementById('toggle-transitions');
    const content = document.getElementById('transitions-content');
    const icon = document.getElementById('toggle-icon');

    if (toggleBtn && content && icon) {
      toggleBtn.addEventListener('click', () => {
        const isHidden = content.classList.contains('hidden');

        if (isHidden) {
          content.classList.remove('hidden');
          icon.style.transform = 'rotate(180deg)';
        } else {
          content.classList.add('hidden');
          icon.style.transform = 'rotate(0deg)';
        }
      });
    }
  }

  /**
   * Actualiza la visualización
   */
  update(): void {
    this.render();
  }
}
