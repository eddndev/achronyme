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
 * Estado de la búsqueda BFS
 */
interface BFSState {
  queue: SearchNode[];
  visited: Set<string>;
  currentNode: SearchNode | null;
  tree: SearchNode | null;
  path: Position[] | null;
  finished: boolean;
  foundGoal: boolean;
}

/**
 * Modo de ejecución
 */
type ExecutionMode = 'idle' | 'step' | 'auto';

/**
 * Componente de visualización del algoritmo BFS
 */
export class BFSVisualization {
  private container: HTMLElement;
  private grid: Grid;
  private state: BFSState;
  private stateHistory: BFSState[] = []; // Historial de estados
  private mode: ExecutionMode = 'idle';
  private animationSpeed: number = 1000; // ms
  private animationInterval: number | null = null;
  private treeContainer: HTMLElement | null = null;
  private queueContainer: HTMLElement | null = null;
  private svg: d3.Selection<SVGSVGElement, unknown, null, undefined> | null = null;
  private g: d3.Selection<SVGGElement, unknown, null, undefined> | null = null;

  constructor(containerId: string, grid: Grid) {
    // El containerId ya no es necesario porque los contenedores están en el DOM de Blade
    // Pero lo mantenemos por compatibilidad
    this.container = document.body; // Usamos body como fallback
    this.grid = grid;
    this.state = this.createInitialState();

    // Esperar a que el DOM esté listo antes de inicializar
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => this.render());
    } else {
      this.render();
    }
  }

  /**
   * Crea el estado inicial de BFS
   */
  private createInitialState(): BFSState {
    return {
      queue: [],
      visited: new Set(),
      currentNode: null,
      tree: null,
      path: null,
      finished: false,
      foundGoal: false
    };
  }

  /**
   * Renderiza el componente principal
   */
  render(): void {
    // Los controles y la estructura HTML están ahora en Blade
    // Solo necesitamos obtener las referencias a los contenedores
    this.treeContainer = document.getElementById('tree-visualization');
    this.queueContainer = document.getElementById('queue-visualization');

    // Inicializar el objeto reactivo de Alpine.js para la cola
    if (typeof window !== 'undefined') {
      (window as any).bfsQueueData = (window as any).Alpine?.reactive({ queue: [] }) || { queue: [] };
    }

    console.log('[BFSVisualization] Initialized');
    this.attachEventListeners();
  }

  /**
   * Inicializa BFS
   */
  private initializeBFS(): void {
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

    // Crear nodo raíz
    const rootNode: SearchNode = {
      id: `${startPos.row},${startPos.col}`,
      position: startPos,
      parent: null,
      depth: 0,
      children: []
    };

    this.state = {
      queue: [rootNode],
      visited: new Set([rootNode.id]),
      currentNode: null,
      tree: rootNode,
      path: null,
      finished: false,
      foundGoal: false
    };

    // Limpiar historial y guardar estado inicial
    this.stateHistory = [];
    this.saveStateToHistory();

    this.updateStatus('BFS iniciado. Explorando...');
    this.showPostInitButtons();
    this.updateVisualization();
    this.updateNavigationButtons();
  }

  /**
   * Ejecuta un paso del algoritmo BFS
   */
  private stepBFS(): boolean {
    if (this.state.finished || this.state.queue.length === 0) {
      this.state.finished = true;
      if (!this.state.foundGoal) {
        this.updateStatus('❌ No se encontró un camino al objetivo');
      }
      this.stopAutoMode();
      return false;
    }

    // Guardar estado antes de ejecutar el paso
    this.saveStateToHistory();

    // Dequeue
    const current = this.state.queue.shift()!;
    this.state.currentNode = current;

    // Verificar si es objetivo
    const goalPositions = this.grid.getGoalPositions();
    const isGoal = goalPositions.some(
      goal => goal.row === current.position.row && goal.col === current.position.col
    );

    if (isGoal) {
      this.state.finished = true;
      this.state.foundGoal = true;
      this.state.path = this.reconstructPath(current);
      this.updateStatus(`✅ ¡Objetivo encontrado! Camino de longitud ${this.state.path.length - 1}`);
      this.stopAutoMode();
      this.updateVisualization();
      this.updateNavigationButtons();
      return false;
    }

    // Explorar vecinos
    const neighbors = this.getValidNeighbors(current.position);
    for (const neighbor of neighbors) {
      const neighborId = `${neighbor.row},${neighbor.col}`;

      if (!this.state.visited.has(neighborId)) {
        const neighborNode: SearchNode = {
          id: neighborId,
          position: neighbor,
          parent: current,
          depth: current.depth + 1,
          children: []
        };

        current.children.push(neighborNode);
        this.state.queue.push(neighborNode);
        this.state.visited.add(neighborId);
      }
    }

    this.updateStatus(`Explorando nodo (${current.position.row + 1}, ${current.position.col + 1})`);
    this.updateVisualization();
    this.updateNavigationButtons();
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
    const statusEl = document.getElementById('bfs-status');
    if (statusEl) {
      statusEl.innerHTML = `
        <div class="text-center font-medium text-slate-700 dark:text-slate-300">${message}</div>
      `;
    }
  }

  /**
   * Actualiza todas las visualizaciones
   */
  private updateVisualization(): void {
    this.updateQueueVisualization();
    this.updateTreeVisualization();
    this.updateStatistics();
  }

  /**
   * Actualiza la visualización de la cola usando Alpine.js
   */
  private updateQueueVisualization(): void {
    // Preparar los datos de la cola para Alpine.js
    const queueData = this.state.queue.map(node => ({
      id: node.id,
      position: node.position,
      depth: node.depth
    }));

    // Actualizar el objeto reactivo de Alpine.js
    if (typeof window !== 'undefined') {
      if (!(window as any).bfsQueueData) {
        (window as any).bfsQueueData = (window as any).Alpine?.reactive({ queue: [] }) || { queue: [] };
      }
      (window as any).bfsQueueData.queue = queueData;
    }
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
      .attr('viewBox', `0 0 ${width} ${height}`)
      .style('cursor', 'grab');

    // Crear grupo principal para zoom/pan
    this.g = this.svg.append('g')
      .attr('transform', 'translate(50, 50)');

    // Configurar zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4]) // Límites de zoom: 10% a 400%
      .on('zoom', (event) => {
        this.g?.attr('transform', event.transform);
      });

    // Aplicar zoom al SVG
    this.svg.call(zoom as any)
      .on('dblclick.zoom', null); // Deshabilitar zoom con doble clic

    // Cambiar cursor durante el drag
    this.svg.on('mousedown.cursor', () => {
      this.svg?.style('cursor', 'grabbing');
    });
    this.svg.on('mouseup.cursor', () => {
      this.svg?.style('cursor', 'grab');
    });

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
    // Si es parte del camino solución
    if (this.state.path) {
      const isInPath = this.state.path.some(
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

    // Si está en la cola
    const inQueue = this.state.queue.some(n => n.id === node.id);
    if (inQueue) return '#d1d5db'; // gray-300

    // Nodo visitado
    return '#60a5fa'; // blue-400
  }

  /**
   * Obtiene el color del borde de un nodo
   */
  private getNodeStroke(node: SearchNode): string {
    if (this.state.path) {
      const isInPath = this.state.path.some(
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

    const inQueue = this.state.queue.some(n => n.id === node.id);
    if (inQueue) return '#6b7280'; // gray-500

    return '#2563eb'; // blue-600
  }

  /**
   * Clona un nodo del árbol de búsqueda de forma recursiva
   */
  private cloneSearchNode(node: SearchNode | null): SearchNode | null {
    if (!node) return null;

    const clonedNode: SearchNode = {
      id: node.id,
      position: { ...node.position },
      parent: null, // Se establecerá después
      depth: node.depth,
      children: []
    };

    // Clonar hijos recursivamente
    for (const child of node.children) {
      const clonedChild = this.cloneSearchNode(child);
      if (clonedChild) {
        clonedChild.parent = clonedNode;
        clonedNode.children.push(clonedChild);
      }
    }

    return clonedNode;
  }

  /**
   * Encuentra un nodo en el árbol clonado por su ID
   */
  private findNodeInTree(tree: SearchNode | null, nodeId: string): SearchNode | null {
    if (!tree) return null;
    if (tree.id === nodeId) return tree;

    for (const child of tree.children) {
      const found = this.findNodeInTree(child, nodeId);
      if (found) return found;
    }

    return null;
  }

  /**
   * Guarda el estado actual en el historial
   */
  private saveStateToHistory(): void {
    // Clonar el árbol
    const clonedTree = this.cloneSearchNode(this.state.tree);

    // Clonar la cola
    const clonedQueue: SearchNode[] = [];
    for (const queueNode of this.state.queue) {
      // Buscar el nodo correspondiente en el árbol clonado
      const nodeInClonedTree = this.findNodeInTree(clonedTree, queueNode.id);
      if (nodeInClonedTree) {
        clonedQueue.push(nodeInClonedTree);
      }
    }

    // Clonar el nodo actual
    let clonedCurrentNode: SearchNode | null = null;
    if (this.state.currentNode) {
      clonedCurrentNode = this.findNodeInTree(clonedTree, this.state.currentNode.id);
    }

    // Guardar estado clonado
    const clonedState: BFSState = {
      queue: clonedQueue,
      visited: new Set(this.state.visited),
      currentNode: clonedCurrentNode,
      tree: clonedTree,
      path: this.state.path ? [...this.state.path.map(pos => ({ ...pos }))] : null,
      finished: this.state.finished,
      foundGoal: this.state.foundGoal
    };

    this.stateHistory.push(clonedState);

    // Limitar el historial a 100 estados para evitar problemas de memoria
    if (this.stateHistory.length > 100) {
      this.stateHistory.shift();
    }
  }

  /**
   * Retrocede al estado anterior
   */
  private previousStep(): void {
    if (this.stateHistory.length <= 1) {
      console.log('[BFSVisualization] No hay paso anterior disponible');
      return;
    }

    // Eliminar el estado actual
    this.stateHistory.pop();

    // Obtener el estado anterior
    const previousState = this.stateHistory[this.stateHistory.length - 1];

    // Restaurar el estado
    this.state = {
      queue: [...previousState.queue],
      visited: new Set(previousState.visited),
      currentNode: previousState.currentNode,
      tree: previousState.tree,
      path: previousState.path ? [...previousState.path] : null,
      finished: previousState.finished,
      foundGoal: previousState.foundGoal
    };

    // Actualizar mensaje de estado
    if (this.state.currentNode) {
      this.updateStatus(`Explorando nodo (${this.state.currentNode.position.row + 1}, ${this.state.currentNode.position.col + 1})`);
    } else {
      this.updateStatus('BFS iniciado. Explorando...');
    }

    // Actualizar visualizaciones
    this.updateVisualization();
    this.updateNavigationButtons();
  }

  /**
   * Actualiza el estado de los botones de navegación
   */
  private updateNavigationButtons(): void {
    const prevBtn = document.getElementById('prev-step-bfs') as HTMLButtonElement;

    if (prevBtn) {
      // Deshabilitar si estamos en el estado inicial
      if (this.stateHistory.length <= 1) {
        prevBtn.disabled = true;
        prevBtn.classList.add('opacity-50', 'cursor-not-allowed');
      } else {
        prevBtn.disabled = false;
        prevBtn.classList.remove('opacity-50', 'cursor-not-allowed');
      }
    }
  }

  /**
   * Actualiza las estadísticas
   */
  private updateStatistics(): void {
    const visitedCountEl = document.getElementById('visited-count');
    if (visitedCountEl) {
      visitedCountEl.textContent = this.state.visited.size.toString();
    }

    const treeDepthEl = document.getElementById('tree-depth');
    if (treeDepthEl && this.state.currentNode) {
      treeDepthEl.textContent = this.state.currentNode.depth.toString();
    }

    const pathLengthEl = document.getElementById('path-length');
    if (pathLengthEl) {
      pathLengthEl.textContent = this.state.path ? (this.state.path.length - 1).toString() : '-';
    }
  }

  /**
   * Muestra los botones correspondientes después de inicializar
   */
  private showPostInitButtons(): void {
    // Ocultar botones de iniciar
    const startStepBtn = document.getElementById('start-bfs-step');
    const startAutoBtn = document.getElementById('start-bfs-auto');
    if (startStepBtn) startStepBtn.classList.add('hidden');
    if (startAutoBtn) startAutoBtn.classList.add('hidden');

    // Obtener el modo actual del radio button
    const executionModeRadios = document.getElementsByName('execution_mode') as NodeListOf<HTMLInputElement>;
    let currentMode: 'step' | 'auto' = 'step';

    for (const radio of executionModeRadios) {
      if (radio.checked) {
        currentMode = radio.value as 'step' | 'auto';
        break;
      }
    }

    if (currentMode === 'step') {
      // Mostrar grupo de botones de control para modo paso a paso
      const stepControlButtons = document.getElementById('step-control-buttons');
      if (stepControlButtons) stepControlButtons.classList.remove('hidden');
    } else {
      // Mostrar grupo de botones de control y velocidad para modo automático
      const autoControlButtons = document.getElementById('auto-control-buttons');
      const speedControl = document.getElementById('speed-control');
      if (autoControlButtons) autoControlButtons.classList.remove('hidden');
      if (speedControl) speedControl.classList.remove('hidden');

      // Iniciar modo automático
      this.startAutoMode();
    }
  }

  /**
   * Inicia el modo automático
   */
  private startAutoMode(): void {
    this.mode = 'auto';

    const pauseBtn = document.getElementById('pause-bfs-auto');
    const resumeBtn = document.getElementById('resume-bfs-auto');

    if (pauseBtn) pauseBtn.classList.remove('hidden');
    if (resumeBtn) resumeBtn.classList.add('hidden');

    this.animationInterval = window.setInterval(() => {
      const shouldContinue = this.stepBFS();
      if (!shouldContinue) {
        this.stopAutoMode();
      }
    }, this.animationSpeed);
  }

  /**
   * Pausa el modo automático
   */
  private pauseAutoMode(): void {
    if (this.animationInterval !== null) {
      clearInterval(this.animationInterval);
      this.animationInterval = null;
    }

    const pauseBtn = document.getElementById('pause-bfs-auto');
    const resumeBtn = document.getElementById('resume-bfs-auto');

    if (pauseBtn) pauseBtn.classList.add('hidden');
    if (resumeBtn) resumeBtn.classList.remove('hidden');
  }

  /**
   * Reanuda el modo automático
   */
  private resumeAutoMode(): void {
    this.startAutoMode();
  }

  /**
   * Detiene el modo automático completamente
   */
  private stopAutoMode(): void {
    if (this.animationInterval !== null) {
      clearInterval(this.animationInterval);
      this.animationInterval = null;
    }

    const pauseBtn = document.getElementById('pause-bfs-auto');
    const resumeBtn = document.getElementById('resume-bfs-auto');

    if (pauseBtn) pauseBtn.classList.add('hidden');
    if (resumeBtn) resumeBtn.classList.add('hidden');
  }

  /**
   * Maneja el cambio de modo de ejecución
   */
  private handleExecutionModeChange(mode: 'step' | 'auto'): void {
    const stepModeButtons = document.getElementById('step-mode-buttons');
    const autoModeButtons = document.getElementById('auto-mode-buttons');

    if (mode === 'step') {
      // Mostrar botones de modo paso a paso
      if (stepModeButtons) stepModeButtons.classList.remove('hidden');
      if (autoModeButtons) autoModeButtons.classList.add('hidden');

      // Detener modo automático si está corriendo
      this.stopAutoMode();
    } else {
      // Mostrar botones de modo automático
      if (stepModeButtons) stepModeButtons.classList.add('hidden');
      if (autoModeButtons) autoModeButtons.classList.remove('hidden');
    }
  }

  /**
   * Reinicia la visualización
   */
  private reset(): void {
    this.stopAutoMode();
    this.state = this.createInitialState();
    this.stateHistory = []; // Limpiar historial

    // Restablecer el mensaje de estado
    this.updateStatus('Presiona "Iniciar BFS" para comenzar la búsqueda');

    // Limpiar visualización de la cola usando Alpine.js
    if (typeof window !== 'undefined' && (window as any).bfsQueueData) {
      (window as any).bfsQueueData.queue = [];
    }

    // Limpiar visualización del árbol
    if (this.treeContainer) {
      this.treeContainer.innerHTML = '<div class="absolute inset-0 flex items-center justify-center text-slate-400 dark:text-slate-500 pointer-events-none">Árbol vacío</div>';
    }

    // Restablecer estadísticas
    const visitedCountEl = document.getElementById('visited-count');
    const treeDepthEl = document.getElementById('tree-depth');
    const pathLengthEl = document.getElementById('path-length');
    if (visitedCountEl) visitedCountEl.textContent = '0';
    if (treeDepthEl) treeDepthEl.textContent = '0';
    if (pathLengthEl) pathLengthEl.textContent = '-';

    // Ocultar todos los grupos de botones post-init y mostrar botones de inicio
    const startStepBtn = document.getElementById('start-bfs-step');
    const startAutoBtn = document.getElementById('start-bfs-auto');
    const stepControlButtons = document.getElementById('step-control-buttons');
    const autoControlButtons = document.getElementById('auto-control-buttons');
    const speedControl = document.getElementById('speed-control');

    if (startStepBtn) startStepBtn.classList.remove('hidden');
    if (startAutoBtn) startAutoBtn.classList.remove('hidden');
    if (stepControlButtons) stepControlButtons.classList.add('hidden');
    if (autoControlButtons) autoControlButtons.classList.add('hidden');
    if (speedControl) speedControl.classList.add('hidden');
  }

  /**
   * Adjunta event listeners
   */
  private attachEventListeners(): void {
    // Botones de inicio para cada modo
    const startStepBtn = document.getElementById('start-bfs-step');
    const startAutoBtn = document.getElementById('start-bfs-auto');
    startStepBtn?.addEventListener('click', () => this.initializeBFS());
    startAutoBtn?.addEventListener('click', () => this.initializeBFS());

    // Botones de navegación (modo paso a paso)
    const nextStepBtn = document.getElementById('next-step-bfs');
    const prevStepBtn = document.getElementById('prev-step-bfs');
    nextStepBtn?.addEventListener('click', () => this.stepBFS());
    prevStepBtn?.addEventListener('click', () => this.previousStep());

    // Botones de control modo automático
    const pauseAutoBtn = document.getElementById('pause-bfs-auto');
    const resumeAutoBtn = document.getElementById('resume-bfs-auto');
    pauseAutoBtn?.addEventListener('click', () => this.pauseAutoMode());
    resumeAutoBtn?.addEventListener('click', () => this.resumeAutoMode());

    // Botones de reinicio para cada modo
    const resetStepBtn = document.getElementById('reset-bfs-step');
    const resetAutoBtn = document.getElementById('reset-bfs-auto');
    resetStepBtn?.addEventListener('click', () => this.reset());
    resetAutoBtn?.addEventListener('click', () => this.reset());

    // Event listener para el radio button de modo de ejecución
    const executionModeRadios = document.getElementsByName('execution_mode');
    executionModeRadios.forEach((radio) => {
      radio.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement;
        if (target.checked) {
          this.handleExecutionModeChange(target.value as 'step' | 'auto');
        }
      });
    });

    // Control de velocidad
    const speedSlider = document.getElementById('speed-slider') as HTMLInputElement;
    const speedValue = document.getElementById('speed-value');

    speedSlider?.addEventListener('input', (e) => {
      const value = parseInt((e.target as HTMLInputElement).value);
      this.animationSpeed = 2100 - value; // Invertir para que más alto = más rápido
      if (speedValue) {
        speedValue.textContent = `${this.animationSpeed} ms`;
      }

      // Si está en modo auto, reiniciar con nueva velocidad
      if (this.mode === 'auto' && this.animationInterval !== null) {
        this.pauseAutoMode();
        this.resumeAutoMode();
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
