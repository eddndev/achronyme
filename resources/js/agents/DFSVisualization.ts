import * as d3 from 'd3';
import { Grid } from './Grid';
import { Position } from './types';

/**
 * Nodo del árbol de búsqueda
 */
interface SearchNode {
  id: string;
  position: Position;
  parent: SearchNode | null;
  depth: number;
  x?: number;
  y?: number;
  children: SearchNode[];
}

/**
 * Estado de la búsqueda DFS
 */
interface DFSState {
  stack: SearchNode[];
  currentNode: SearchNode | null;
  tree: SearchNode | null;
  solutions: Position[][];
  finished: boolean;
  maxSolutions: number;
  nodesExplored: number;
}

/**
 * Modo de ejecución
 */
type ExecutionMode = 'idle' | 'step' | 'auto';

/**
 * Componente de visualización del algoritmo DFS
 */
export class DFSVisualization {
  private container: HTMLElement;
  private grid: Grid;
  private state: DFSState;
  private mode: ExecutionMode = 'idle';
  private animationSpeed: number = 1000;
  private animationInterval: number | null = null;
  private treeContainer: HTMLElement | null = null;
  private stackContainer: HTMLElement | null = null;
  private svg: d3.Selection<SVGSVGElement, unknown, null, undefined> | null = null;
  private g: d3.Selection<SVGGElement, unknown, null, undefined> | null = null;

  constructor(containerId: string, grid: Grid) {
    const container = document.getElementById(containerId);
    if (!container) {
      throw new Error(`Container with id "${containerId}" not found`);
    }
    this.container = container;
    this.grid = grid;
    this.state = this.createInitialState();
    this.render();
  }

  /**
   * Crea el estado inicial de DFS
   */
  private createInitialState(): DFSState {
    return {
      stack: [],
      currentNode: null,
      tree: null,
      solutions: [],
      finished: false,
      maxSolutions: 1,
      nodesExplored: 0
    };
  }

  /**
   * Renderiza el componente principal
   */
  render(): void {
    this.container.innerHTML = `
      <div class="bg-white rounded-lg shadow-lg p-6 space-y-6">
        <!-- Título -->
        <div class="border-b pb-4">
          <h2 class="text-2xl font-bold text-gray-800">Algoritmo DFS - Búsqueda en Profundidad</h2>
          <p class="text-sm text-gray-600 mt-1">Visualización paso a paso con búsqueda de múltiples soluciones</p>
        </div>

        <!-- Estado del algoritmo -->
        <div id="dfs-status" class="bg-gray-100 p-4 rounded-md border-2 border-gray-300">
          <div class="text-center text-gray-600">
            Configura el número de soluciones y presiona "Iniciar DFS"
          </div>
        </div>

        <!-- Controles -->
        <div class="space-y-3">
          <!-- Número de soluciones -->
          <div class="flex items-center gap-3 bg-purple-50 p-3 rounded-md border border-purple-200">
            <label class="text-sm font-medium text-gray-700">Soluciones a buscar:</label>
            <input type="number" id="max-solutions" min="1" max="10" value="1" class="w-20 px-3 py-1 border border-gray-300 rounded-md" />
            <span class="text-xs text-gray-500">(-1 para buscar todas)</span>
          </div>

          <div class="flex gap-2 flex-wrap items-center">
            <button id="start-dfs" class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition-colors text-sm font-medium">
              ▶️ Iniciar DFS
            </button>
            <button id="step-dfs" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors text-sm font-medium" disabled>
              ⏯️ Paso a Paso
            </button>
            <button id="play-dfs" class="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 transition-colors text-sm font-medium" disabled>
              ▶️ Auto
            </button>
            <button id="pause-dfs" class="px-4 py-2 bg-orange-600 text-white rounded-md hover:bg-orange-700 transition-colors text-sm font-medium hidden">
              ⏸️ Pausar
            </button>
            <button id="reset-dfs" class="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 transition-colors text-sm font-medium">
              🔄 Reiniciar
            </button>
          </div>

          <!-- Control de velocidad -->
          <div class="flex items-center gap-3 bg-gray-50 p-3 rounded-md">
            <label class="text-sm font-medium text-gray-700">Velocidad:</label>
            <input type="range" id="speed-slider-dfs" min="100" max="2000" value="1000" step="100" class="flex-1" />
            <span id="speed-value-dfs" class="text-sm font-semibold text-gray-700 min-w-[80px]">1000 ms</span>
          </div>
        </div>

        <!-- Visualización de la Pila -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-gray-700 flex items-center gap-2">
            <span class="text-orange-600">📚 Pila (Stack):</span>
            <span id="stack-count" class="text-sm bg-orange-100 px-2 py-1 rounded">0 nodos</span>
          </h3>
          <div id="stack-visualization" class="bg-orange-50 p-4 rounded-md border-2 border-orange-200 min-h-[200px] overflow-y-auto">
            <div class="text-gray-400 text-center text-sm">La pila está vacía</div>
          </div>
        </div>

        <!-- Soluciones Encontradas -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-gray-700 flex items-center gap-2">
            <span class="text-green-600">✅ Soluciones Encontradas:</span>
            <span id="solutions-count" class="text-sm bg-green-100 px-2 py-1 rounded">0</span>
          </h3>
          <div id="solutions-list" class="bg-green-50 p-4 rounded-md border-2 border-green-200 max-h-[200px] overflow-y-auto">
            <div class="text-gray-400 text-center text-sm">No se han encontrado soluciones</div>
          </div>
        </div>

        <!-- Estadísticas -->
        <div class="grid grid-cols-3 gap-4">
          <div class="bg-blue-50 p-4 rounded-md border border-blue-200">
            <div class="text-2xl font-bold text-blue-600" id="visited-count-dfs">0</div>
            <div class="text-xs text-gray-600">Nodos Visitados</div>
          </div>
          <div class="bg-purple-50 p-4 rounded-md border border-purple-200">
            <div class="text-2xl font-bold text-purple-600" id="tree-depth-dfs">0</div>
            <div class="text-xs text-gray-600">Profundidad Máxima</div>
          </div>
          <div class="bg-green-50 p-4 rounded-md border border-green-200">
            <div class="text-2xl font-bold text-green-600" id="shortest-path-dfs">-</div>
            <div class="text-xs text-gray-600">Camino Más Corto</div>
          </div>
        </div>

        <!-- Árbol de Búsqueda -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-gray-700">🌳 Árbol de Búsqueda DFS</h3>
          <div id="tree-visualization-dfs" class="border-2 border-gray-300 rounded-lg bg-gray-50 overflow-auto" style="height: 600px;">
            <div class="flex items-center justify-center h-full text-gray-400">
              Árbol vacío
            </div>
          </div>
        </div>

        <!-- Leyenda -->
        <div class="bg-gray-50 p-4 rounded-md">
          <h3 class="text-sm font-semibold text-gray-700 mb-2">Leyenda:</h3>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs">
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded-full bg-green-500 border-2 border-green-700"></div>
              <span>Nodo Inicial</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded-full bg-red-500 border-2 border-red-700"></div>
              <span>Nodo Objetivo</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded-full bg-yellow-400 border-2 border-yellow-600"></div>
              <span>Nodo Actual</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded-full bg-blue-400 border-2 border-blue-600"></div>
              <span>Nodo Visitado</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded-full bg-orange-300 border-2 border-orange-500"></div>
              <span>Nodo en Pila</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded-full bg-purple-500 border-2 border-purple-700"></div>
              <span>Camino Solución</span>
            </div>
          </div>
        </div>
      </div>
    `;

    this.treeContainer = document.getElementById('tree-visualization-dfs');
    this.stackContainer = document.getElementById('stack-visualization');
    this.attachEventListeners();
  }

  /**
   * Inicializa DFS
   */
  private initializeDFS(): void {
    const startPos = this.grid.getStartPosition();
    if (!startPos) {
      alert('Por favor, establece una posición de inicio primero');
      return;
    }

    const goalPositions = this.grid.getGoalPositions();
    if (goalPositions.length === 0) {
      alert('Por favor, establece al menos un destino primero');
      return;
    }

    // Obtener número de soluciones
    const maxSolutionsInput = document.getElementById('max-solutions') as HTMLInputElement;
    const maxSolutions = parseInt(maxSolutionsInput.value);

    // Crear nodo raíz
    const rootNode: SearchNode = {
      id: `${startPos.row},${startPos.col}`,
      position: startPos,
      parent: null,
      depth: 0,
      children: []
    };

    this.state = {
      stack: [rootNode],
      currentNode: null,
      tree: rootNode,
      solutions: [],
      finished: false,
      maxSolutions: maxSolutions,
      nodesExplored: 0
    };

    this.updateStatus(`DFS iniciado. Buscando ${maxSolutions === -1 ? 'todas las' : maxSolutions} solución(es).`);
    this.enableButtons();
    this.updateVisualization();
  }

  /**
   * Verifica si un nodo está en el camino actual (desde la raíz hasta el nodo)
   */
  private isInCurrentPath(nodeId: string, node: SearchNode): boolean {
    let current: SearchNode | null = node;
    while (current !== null) {
      if (current.id === nodeId) {
        return true;
      }
      current = current.parent;
    }
    return false;
  }

  /**
   * Ejecuta un paso del algoritmo DFS
   */
  private stepDFS(): boolean {
    if (this.state.finished || this.state.stack.length === 0) {
      this.state.finished = true;
      const msg = this.state.solutions.length > 0
        ? `✅ Búsqueda completada. Se encontraron ${this.state.solutions.length} solución(es).`
        : '❌ No se encontró ninguna solución';
      this.updateStatus(msg);
      this.stopAutoMode();
      return false;
    }

    // Pop (LIFO)
    const current = this.state.stack.pop()!;
    this.state.currentNode = current;
    this.state.nodesExplored++;

    // Verificar si es objetivo
    const goalPositions = this.grid.getGoalPositions();
    const isGoal = goalPositions.some(
      goal => goal.row === current.position.row && goal.col === current.position.col
    );

    if (isGoal) {
      const path = this.reconstructPath(current);
      this.state.solutions.push(path);

      this.updateStatus(
        `🎯 Solución ${this.state.solutions.length} encontrada! Longitud: ${path.length - 1}`
      );

      // Verificar si debemos seguir buscando
      if (
        this.state.maxSolutions !== -1 &&
        this.state.solutions.length >= this.state.maxSolutions
      ) {
        this.state.finished = true;
        this.updateStatus(
          `✅ Búsqueda completada. Se encontraron ${this.state.solutions.length} solución(es).`
        );
        this.stopAutoMode();
        this.updateVisualization();
        return false;
      }

      // No explorar más desde este nodo objetivo
      this.updateVisualization();
      return true;
    }

    // Explorar vecinos (en orden inverso para que la pila procese en el orden correcto)
    const neighbors = this.getValidNeighbors(current.position).reverse();

    for (const neighbor of neighbors) {
      const neighborId = `${neighbor.row},${neighbor.col}`;

      // Verificar que el vecino NO esté en el camino actual (evitar ciclos en la misma rama)
      if (!this.isInCurrentPath(neighborId, current)) {
        const neighborNode: SearchNode = {
          id: neighborId,
          position: neighbor,
          parent: current,
          depth: current.depth + 1,
          children: []
        };

        current.children.push(neighborNode);
        this.state.stack.push(neighborNode);
      }
    }

    this.updateStatus(`Explorando nodo (${current.position.row + 1}, ${current.position.col + 1})`);
    this.updateVisualization();
    return true;
  }

  /**
   * Obtiene vecinos válidos de una posición
   */
  private getValidNeighbors(pos: Position): Position[] {
    const config = this.grid.getConfig();
    const neighbors: Position[] = [];
    const directions = [
      { row: -1, col: 0 }, // Arriba
      { row: 1, col: 0 },  // Abajo
      { row: 0, col: -1 }, // Izquierda
      { row: 0, col: 1 }   // Derecha
    ];

    for (const dir of directions) {
      const newRow = pos.row + dir.row;
      const newCol = pos.col + dir.col;

      if (
        newRow >= 0 && newRow < config.rows &&
        newCol >= 0 && newCol < config.cols
      ) {
        const cell = this.grid.getCell(newRow, newCol);
        if (cell && cell.type !== 'obstacle') {
          neighbors.push({ row: newRow, col: newCol });
        }
      }
    }

    return neighbors;
  }

  /**
   * Reconstruye el camino desde el nodo hasta la raíz
   */
  private reconstructPath(node: SearchNode): Position[] {
    const path: Position[] = [];
    let current: SearchNode | null = node;

    while (current !== null) {
      path.unshift(current.position);
      current = current.parent;
    }

    return path;
  }

  /**
   * Actualiza el mensaje de estado
   */
  private updateStatus(message: string): void {
    const statusEl = document.getElementById('dfs-status');
    if (statusEl) {
      statusEl.innerHTML = `
        <div class="text-center font-medium text-gray-700">${message}</div>
      `;
    }
  }

  /**
   * Actualiza todas las visualizaciones
   */
  private updateVisualization(): void {
    this.updateStackVisualization();
    this.updateSolutionsVisualization();
    this.updateTreeVisualization();
    this.updateStatistics();
  }

  /**
   * Actualiza la visualización de la pila
   */
  private updateStackVisualization(): void {
    if (!this.stackContainer) return;

    const stackCountEl = document.getElementById('stack-count');
    if (stackCountEl) {
      stackCountEl.textContent = `${this.state.stack.length} nodos`;
    }

    if (this.state.stack.length === 0) {
      this.stackContainer.innerHTML = '<div class="text-gray-400 text-center text-sm">La pila está vacía</div>';
      return;
    }

    // Mostrar pila de arriba (último) hacia abajo (primero)
    const stackHTML = [...this.state.stack].reverse().map((node, index) => `
      <div class="flex items-center gap-3 py-2 px-3 mb-2 bg-white rounded-md border-2 ${
        index === 0 ? 'border-orange-500' : 'border-orange-200'
      }">
        <div class="w-12 h-12 rounded-full bg-orange-300 border-2 border-orange-500 flex items-center justify-center flex-shrink-0">
          <span class="font-mono text-xs font-bold">(${node.position.row + 1},${node.position.col + 1})</span>
        </div>
        <div class="flex-1">
          <div class="text-xs text-gray-600">Profundidad: ${node.depth}</div>
          ${index === 0 ? '<div class="text-xs font-semibold text-orange-600">← Próximo a explorar (TOP)</div>' : ''}
        </div>
      </div>
    `).join('');

    this.stackContainer.innerHTML = stackHTML;
  }

  /**
   * Actualiza la visualización de soluciones
   */
  private updateSolutionsVisualization(): void {
    const solutionsListEl = document.getElementById('solutions-list');
    const solutionsCountEl = document.getElementById('solutions-count');

    if (solutionsCountEl) {
      solutionsCountEl.textContent = this.state.solutions.length.toString();
    }

    if (!solutionsListEl) return;

    if (this.state.solutions.length === 0) {
      solutionsListEl.innerHTML = '<div class="text-gray-400 text-center text-sm">No se han encontrado soluciones</div>';
      return;
    }

    const solutionsHTML = this.state.solutions.map((path, index) => `
      <div class="mb-3 p-3 bg-white rounded-md border border-green-300">
        <div class="font-semibold text-green-700 mb-1">Solución ${index + 1} - Longitud: ${path.length - 1}</div>
        <div class="text-xs font-mono text-gray-600">
          ${path.map(pos => `(${pos.row + 1},${pos.col + 1})`).join(' → ')}
        </div>
      </div>
    `).join('');

    solutionsListEl.innerHTML = solutionsHTML;
  }

  /**
   * Actualiza la visualización del árbol
   */
  private updateTreeVisualization(): void {
    if (!this.treeContainer || !this.state.tree) return;

    const width = 1400;
    const height = 800;

    // Limpiar SVG anterior
    if (this.svg) {
      this.svg.remove();
    }

    // Crear SVG
    this.svg = d3.select(this.treeContainer)
      .html('')
      .append('svg')
      .attr('width', '100%')
      .attr('height', '100%')
      .attr('viewBox', `0 0 ${width} ${height}`);

    this.g = this.svg.append('g')
      .attr('transform', 'translate(50, 50)');

    // Configurar layout de árbol
    const treeLayout = d3.tree<SearchNode>()
      .size([width - 100, height - 100])
      .separation((a, b) => (a.parent === b.parent ? 1 : 1.2));

    const root = d3.hierarchy(this.state.tree, d => d.children);
    treeLayout(root);

    // Dibujar enlaces
    this.g.selectAll('.link')
      .data(root.links())
      .enter()
      .append('path')
      .attr('class', 'link')
      .attr('d', d3.linkVertical<any, any>()
        .x(d => d.x)
        .y(d => d.y))
      .attr('fill', 'none')
      .attr('stroke', '#cbd5e1')
      .attr('stroke-width', 2);

    // Dibujar nodos
    const nodes = this.g.selectAll('.node')
      .data(root.descendants())
      .enter()
      .append('g')
      .attr('class', 'node')
      .attr('transform', d => `translate(${d.x},${d.y})`);

    // Círculos de los nodos
    nodes.append('circle')
      .attr('r', 18)
      .attr('fill', d => this.getNodeColor(d.data))
      .attr('stroke', d => this.getNodeStroke(d.data))
      .attr('stroke-width', 3);

    // Etiquetas
    nodes.append('text')
      .attr('dy', -25)
      .attr('text-anchor', 'middle')
      .attr('font-size', '11px')
      .attr('font-weight', 'bold')
      .attr('fill', '#1f2937')
      .attr('stroke', '#ffffff')
      .attr('stroke-width', '3px')
      .attr('paint-order', 'stroke')
      .text(d => `(${d.data.position.row + 1},${d.data.position.col + 1})`);

    // Mostrar profundidad
    nodes.append('text')
      .attr('dy', 5)
      .attr('text-anchor', 'middle')
      .attr('font-size', '9px')
      .attr('fill', '#ffffff')
      .attr('font-weight', 'bold')
      .text(d => `d:${d.data.depth}`);
  }

  /**
   * Obtiene el color de un nodo según su estado
   */
  private getNodeColor(node: SearchNode): string {
    // Si es parte de alguna solución
    for (const solution of this.state.solutions) {
      const isInPath = solution.some(
        pos => pos.row === node.position.row && pos.col === node.position.col
      );
      if (isInPath) return '#a855f7'; // purple-500
    }

    // Si es el nodo actual
    if (this.state.currentNode && this.state.currentNode.id === node.id) {
      return '#facc15'; // yellow-400
    }

    // Si es el nodo inicial
    if (node.parent === null) {
      return '#22c55e'; // green-500
    }

    // Si es un nodo objetivo
    const goalPositions = this.grid.getGoalPositions();
    const isGoal = goalPositions.some(
      goal => goal.row === node.position.row && goal.col === node.position.col
    );
    if (isGoal) return '#ef4444'; // red-500

    // Si está en la pila
    const inStack = this.state.stack.some(n => n.id === node.id);
    if (inStack) return '#fdba74'; // orange-300

    // Nodo visitado
    return '#60a5fa'; // blue-400
  }

  /**
   * Obtiene el color del borde de un nodo
   */
  private getNodeStroke(node: SearchNode): string {
    for (const solution of this.state.solutions) {
      const isInPath = solution.some(
        pos => pos.row === node.position.row && pos.col === node.position.col
      );
      if (isInPath) return '#7c3aed'; // purple-700
    }

    if (this.state.currentNode && this.state.currentNode.id === node.id) {
      return '#ca8a04'; // yellow-600
    }

    if (node.parent === null) return '#16a34a'; // green-600

    const goalPositions = this.grid.getGoalPositions();
    const isGoal = goalPositions.some(
      goal => goal.row === node.position.row && goal.col === node.position.col
    );
    if (isGoal) return '#dc2626'; // red-600

    const inStack = this.state.stack.some(n => n.id === node.id);
    if (inStack) return '#f97316'; // orange-500

    return '#2563eb'; // blue-600
  }

  /**
   * Actualiza las estadísticas
   */
  private updateStatistics(): void {
    const visitedCountEl = document.getElementById('visited-count-dfs');
    if (visitedCountEl) {
      visitedCountEl.textContent = this.state.nodesExplored.toString();
    }

    const treeDepthEl = document.getElementById('tree-depth-dfs');
    if (treeDepthEl && this.state.currentNode) {
      treeDepthEl.textContent = this.state.currentNode.depth.toString();
    }

    const shortestPathEl = document.getElementById('shortest-path-dfs');
    if (shortestPathEl) {
      if (this.state.solutions.length > 0) {
        const shortest = Math.min(...this.state.solutions.map(s => s.length - 1));
        shortestPathEl.textContent = shortest.toString();
      } else {
        shortestPathEl.textContent = '-';
      }
    }
  }

  /**
   * Habilita los botones de control
   */
  private enableButtons(): void {
    const stepBtn = document.getElementById('step-dfs') as HTMLButtonElement;
    const playBtn = document.getElementById('play-dfs') as HTMLButtonElement;

    if (stepBtn) stepBtn.disabled = false;
    if (playBtn) playBtn.disabled = false;
  }

  /**
   * Inicia el modo automático
   */
  private startAutoMode(): void {
    this.mode = 'auto';

    const playBtn = document.getElementById('play-dfs');
    const pauseBtn = document.getElementById('pause-dfs');

    if (playBtn) playBtn.classList.add('hidden');
    if (pauseBtn) pauseBtn.classList.remove('hidden');

    this.animationInterval = window.setInterval(() => {
      const shouldContinue = this.stepDFS();
      if (!shouldContinue) {
        this.stopAutoMode();
      }
    }, this.animationSpeed);
  }

  /**
   * Detiene el modo automático
   */
  private stopAutoMode(): void {
    this.mode = 'step';

    if (this.animationInterval !== null) {
      clearInterval(this.animationInterval);
      this.animationInterval = null;
    }

    const playBtn = document.getElementById('play-dfs');
    const pauseBtn = document.getElementById('pause-dfs');

    if (playBtn) playBtn.classList.remove('hidden');
    if (pauseBtn) pauseBtn.classList.add('hidden');
  }

  /**
   * Reinicia la visualización
   */
  private reset(): void {
    this.stopAutoMode();
    this.state = this.createInitialState();
    this.render();
  }

  /**
   * Adjunta event listeners
   */
  private attachEventListeners(): void {
    const startBtn = document.getElementById('start-dfs');
    startBtn?.addEventListener('click', () => this.initializeDFS());

    const stepBtn = document.getElementById('step-dfs');
    stepBtn?.addEventListener('click', () => this.stepDFS());

    const playBtn = document.getElementById('play-dfs');
    playBtn?.addEventListener('click', () => this.startAutoMode());

    const pauseBtn = document.getElementById('pause-dfs');
    pauseBtn?.addEventListener('click', () => this.stopAutoMode());

    const resetBtn = document.getElementById('reset-dfs');
    resetBtn?.addEventListener('click', () => this.reset());

    const speedSlider = document.getElementById('speed-slider-dfs') as HTMLInputElement;
    const speedValue = document.getElementById('speed-value-dfs');

    speedSlider?.addEventListener('input', (e) => {
      const value = parseInt((e.target as HTMLInputElement).value);
      this.animationSpeed = 2100 - value;
      if (speedValue) {
        speedValue.textContent = `${this.animationSpeed} ms`;
      }

      if (this.mode === 'auto') {
        this.stopAutoMode();
        this.startAutoMode();
      }
    });
  }

  /**
   * Actualiza la visualización
   */
  update(): void {
    this.reset();
  }
}
