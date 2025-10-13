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
  visited: Set<string>;
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
  private stateHistory: DFSState[] = []; // Historial de estados
  private mode: ExecutionMode = 'idle';
  private animationSpeed: number = 1000; // ms
  private animationInterval: number | null = null;
  private treeContainer: HTMLElement | null = null;
  private stackContainer: HTMLElement | null = null;
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
   * Crea el estado inicial de DFS
   */
  private createInitialState(): DFSState {
    return {
      stack: [],
      visited: new Set(),
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
    // Los controles y la estructura HTML están ahora en Blade
    // Solo necesitamos obtener las referencias a los contenedores
    this.treeContainer = document.getElementById('tree-visualization-dfs');
    this.stackContainer = document.getElementById('stack-visualization');

    // Inicializar el objeto global de Alpine.js para la pila
    if (typeof window !== 'undefined' && !(window as any).dfsStackData) {
      (window as any).dfsStackData = (window as any).Alpine?.reactive({ stack: [] }) || { stack: [] };
    }

    console.log('[DFSVisualization] Initialized');
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

    // Leer el número máximo de soluciones del input
    const maxSolutionsInput = document.getElementById('max-solutions-dfs') as HTMLInputElement;
    let maxSolutions = 1;
    if (maxSolutionsInput) {
      const value = parseInt(maxSolutionsInput.value);
      if (!isNaN(value)) {
        maxSolutions = value;
      }
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
      stack: [rootNode],
      visited: new Set([rootNode.id]),
      currentNode: null,
      tree: rootNode,
      solutions: [],
      finished: false,
      maxSolutions: maxSolutions,
      nodesExplored: 0
    };

    // Limpiar historial y guardar estado inicial
    this.stateHistory = [];
    this.saveStateToHistory();

    this.updateStatus(`DFS iniciado. Buscando ${maxSolutions === -1 ? 'todas las soluciones' : `hasta ${maxSolutions} solución(es)`}...`);
    this.showPostInitButtons();
    this.updateVisualization();
    this.updateNavigationButtons();
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

    // Guardar estado antes de ejecutar el paso
    this.saveStateToHistory();

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
        this.updateNavigationButtons();
        return false;
      }

      // No explorar más desde este nodo objetivo
      this.updateVisualization();
      this.updateNavigationButtons();
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
    this.updateTreeVisualization();
    this.updateStatistics();
  }

  /**
   * Actualiza la visualización de la pila
   */
  private updateStackVisualization(): void {
    // Preparar datos de la pila para Alpine.js
    const stackData = this.state.stack.map(node => ({
      id: node.id,
      position: node.position,
      depth: node.depth
    }));

    // Inicializar el objeto global si no existe
    if (typeof window !== 'undefined') {
      if (!(window as any).dfsStackData) {
        (window as any).dfsStackData = (window as any).Alpine?.reactive({ stack: [] }) || { stack: [] };
      }

      // Actualizar el array de la pila
      (window as any).dfsStackData.stack = stackData;

      // Forzar actualización de Alpine.js si está disponible
      if ((window as any).Alpine?.nextTick) {
        (window as any).Alpine.nextTick(() => {
          console.log('[DFSVisualization] Stack updated:', stackData.length, 'nodes');
        });
      }
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
    // Si es el nodo actual (comparación por referencia)
    if (this.state.currentNode && this.state.currentNode === node) {
      return '#facc15'; // yellow-400
    }

    // Si está en la pila (comparación por referencia)
    const inStack = this.state.stack.some(n => n === node);
    if (inStack) return '#64748b'; // slate-500

    // Si es parte de algún camino solución (por posición, porque las soluciones son Position[])
    if (this.state.solutions.length > 0) {
      const isInSolution = this.state.solutions.some(solution =>
        solution.some(pos => pos.row === node.position.row && pos.col === node.position.col)
      );
      if (isInSolution) return '#a855f7'; // purple-500
    }

    // Si es un nodo objetivo (por posición)
    const goalPositions = this.grid.getGoalPositions();
    const isGoal = goalPositions.some(
      goal => goal.row === node.position.row && goal.col === node.position.col
    );
    if (isGoal) return '#ef4444'; // red-500

    // Si es el nodo inicial
    if (node.parent === null) {
      return '#22c55e'; // green-500
    }

    // Nodo visitado (por defecto)
    return '#60a5fa'; // blue-400
  }

  /**
   * Obtiene el color del borde de un nodo
   */
  private getNodeStroke(node: SearchNode): string {
    // Si es el nodo actual (comparación por referencia)
    if (this.state.currentNode && this.state.currentNode === node) {
      return '#ca8a04'; // yellow-600
    }

    // Si está en la pila (comparación por referencia)
    const inStack = this.state.stack.some(n => n === node);
    if (inStack) return '#475569'; // slate-600

    // Si es parte de algún camino solución (por posición)
    if (this.state.solutions.length > 0) {
      const isInSolution = this.state.solutions.some(solution =>
        solution.some(pos => pos.row === node.position.row && pos.col === node.position.col)
      );
      if (isInSolution) return '#7c3aed'; // purple-700
    }

    // Si es un nodo objetivo (por posición)
    const goalPositions = this.grid.getGoalPositions();
    const isGoal = goalPositions.some(
      goal => goal.row === node.position.row && goal.col === node.position.col
    );
    if (isGoal) return '#dc2626'; // red-600

    // Si es el nodo inicial
    if (node.parent === null) return '#16a34a'; // green-600

    // Nodo visitado (por defecto)
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

    const solutionsCountEl = document.getElementById('solutions-count-dfs');
    if (solutionsCountEl) {
      solutionsCountEl.textContent = this.state.solutions.length.toString();
    }

    const pathLengthEl = document.getElementById('path-length-dfs');
    if (pathLengthEl) {
      if (this.state.solutions.length > 0) {
        const shortest = Math.min(...this.state.solutions.map(s => s.length - 1));
        pathLengthEl.textContent = shortest.toString();
      } else {
        pathLengthEl.textContent = '-';
      }
    }
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

    // Clonar la pila
    const clonedStack: SearchNode[] = [];
    for (const stackNode of this.state.stack) {
      // Buscar el nodo correspondiente en el árbol clonado
      const nodeInClonedTree = this.findNodeInTree(clonedTree, stackNode.id);
      if (nodeInClonedTree) {
        clonedStack.push(nodeInClonedTree);
      }
    }

    // Clonar el nodo actual
    let clonedCurrentNode: SearchNode | null = null;
    if (this.state.currentNode) {
      clonedCurrentNode = this.findNodeInTree(clonedTree, this.state.currentNode.id);
    }

    // Clonar las soluciones
    const clonedSolutions: Position[][] = this.state.solutions.map(solution =>
      solution.map(pos => ({ ...pos }))
    );

    // Guardar estado clonado
    const clonedState: DFSState = {
      stack: clonedStack,
      visited: new Set(this.state.visited),
      currentNode: clonedCurrentNode,
      tree: clonedTree,
      solutions: clonedSolutions,
      finished: this.state.finished,
      maxSolutions: this.state.maxSolutions,
      nodesExplored: this.state.nodesExplored
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
      console.log('[DFSVisualization] No hay paso anterior disponible');
      return;
    }

    // Eliminar el estado actual
    this.stateHistory.pop();

    // Obtener el estado anterior
    const previousState = this.stateHistory[this.stateHistory.length - 1];

    // Restaurar el estado
    this.state = {
      stack: [...previousState.stack],
      visited: new Set(previousState.visited),
      currentNode: previousState.currentNode,
      tree: previousState.tree,
      solutions: previousState.solutions.map(solution => [...solution]),
      finished: previousState.finished,
      maxSolutions: previousState.maxSolutions,
      nodesExplored: previousState.nodesExplored
    };

    // Actualizar mensaje de estado
    if (this.state.currentNode) {
      this.updateStatus(`Explorando nodo (${this.state.currentNode.position.row + 1}, ${this.state.currentNode.position.col + 1})`);
    } else {
      this.updateStatus('DFS iniciado. Explorando...');
    }

    // Actualizar visualizaciones
    this.updateVisualization();
    this.updateNavigationButtons();
  }

  /**
   * Actualiza el estado de los botones de navegación
   */
  private updateNavigationButtons(): void {
    const prevBtn = document.getElementById('prev-step-dfs') as HTMLButtonElement;

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
   * Muestra los botones correspondientes después de inicializar
   */
  private showPostInitButtons(): void {
    // Ocultar botones de iniciar
    const startStepBtn = document.getElementById('start-dfs-step');
    const startAutoBtn = document.getElementById('start-dfs-auto');
    if (startStepBtn) startStepBtn.classList.add('hidden');
    if (startAutoBtn) startAutoBtn.classList.add('hidden');

    // Obtener el modo actual del radio button
    const executionModeRadios = document.getElementsByName('execution_mode_dfs') as NodeListOf<HTMLInputElement>;
    let currentMode: 'step' | 'auto' = 'step';

    for (const radio of executionModeRadios) {
      if (radio.checked) {
        currentMode = radio.value as 'step' | 'auto';
        break;
      }
    }

    if (currentMode === 'step') {
      // Mostrar grupo de botones de control para modo paso a paso
      const stepControlButtons = document.getElementById('step-control-buttons-dfs');
      if (stepControlButtons) stepControlButtons.classList.remove('hidden');
    } else {
      // Mostrar grupo de botones de control y velocidad para modo automático
      const autoControlButtons = document.getElementById('auto-control-buttons-dfs');
      const speedControl = document.getElementById('speed-control-dfs');
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

    const pauseBtn = document.getElementById('pause-dfs-auto');
    const resumeBtn = document.getElementById('resume-dfs-auto');

    if (pauseBtn) pauseBtn.classList.remove('hidden');
    if (resumeBtn) resumeBtn.classList.add('hidden');

    this.animationInterval = window.setInterval(() => {
      const shouldContinue = this.stepDFS();
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

    const pauseBtn = document.getElementById('pause-dfs-auto');
    const resumeBtn = document.getElementById('resume-dfs-auto');

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

    const pauseBtn = document.getElementById('pause-dfs-auto');
    const resumeBtn = document.getElementById('resume-dfs-auto');

    if (pauseBtn) pauseBtn.classList.add('hidden');
    if (resumeBtn) resumeBtn.classList.add('hidden');
  }

  /**
   * Maneja el cambio de modo de ejecución
   */
  private handleExecutionModeChange(mode: 'step' | 'auto'): void {
    const stepModeButtons = document.getElementById('step-mode-buttons-dfs');
    const autoModeButtons = document.getElementById('auto-mode-buttons-dfs');

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
    this.updateStatus('Presiona "Iniciar DFS" para comenzar la búsqueda');

    // Limpiar la pila usando Alpine.js
    if (typeof window !== 'undefined' && (window as any).dfsStackData) {
      (window as any).dfsStackData.stack = [];
    }

    // Limpiar visualización del árbol
    if (this.treeContainer) {
      this.treeContainer.innerHTML = '<div class="absolute inset-0 flex items-center justify-center text-slate-400 dark:text-slate-500 pointer-events-none">Árbol vacío</div>';
    }

    // Restablecer estadísticas
    const visitedCountEl = document.getElementById('visited-count-dfs');
    const treeDepthEl = document.getElementById('tree-depth-dfs');
    const solutionsCountEl = document.getElementById('solutions-count-dfs');
    const pathLengthEl = document.getElementById('path-length-dfs');
    if (visitedCountEl) visitedCountEl.textContent = '0';
    if (treeDepthEl) treeDepthEl.textContent = '0';
    if (solutionsCountEl) solutionsCountEl.textContent = '0';
    if (pathLengthEl) pathLengthEl.textContent = '-';

    // Ocultar todos los grupos de botones post-init y mostrar botones de inicio
    const startStepBtn = document.getElementById('start-dfs-step');
    const startAutoBtn = document.getElementById('start-dfs-auto');
    const stepControlButtons = document.getElementById('step-control-buttons-dfs');
    const autoControlButtons = document.getElementById('auto-control-buttons-dfs');
    const speedControl = document.getElementById('speed-control-dfs');

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
    const startStepBtn = document.getElementById('start-dfs-step');
    const startAutoBtn = document.getElementById('start-dfs-auto');
    startStepBtn?.addEventListener('click', () => this.initializeDFS());
    startAutoBtn?.addEventListener('click', () => this.initializeDFS());

    // Botones de navegación (modo paso a paso)
    const nextStepBtn = document.getElementById('next-step-dfs');
    const prevStepBtn = document.getElementById('prev-step-dfs');
    nextStepBtn?.addEventListener('click', () => this.stepDFS());
    prevStepBtn?.addEventListener('click', () => this.previousStep());

    // Botones de control modo automático
    const pauseAutoBtn = document.getElementById('pause-dfs-auto');
    const resumeAutoBtn = document.getElementById('resume-dfs-auto');
    pauseAutoBtn?.addEventListener('click', () => this.pauseAutoMode());
    resumeAutoBtn?.addEventListener('click', () => this.resumeAutoMode());

    // Botones de reinicio para cada modo
    const resetStepBtn = document.getElementById('reset-dfs-step');
    const resetAutoBtn = document.getElementById('reset-dfs-auto');
    resetStepBtn?.addEventListener('click', () => this.reset());
    resetAutoBtn?.addEventListener('click', () => this.reset());

    // Event listener para el radio button de modo de ejecución
    const executionModeRadios = document.getElementsByName('execution_mode_dfs');
    executionModeRadios.forEach((radio) => {
      radio.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement;
        if (target.checked) {
          this.handleExecutionModeChange(target.value as 'step' | 'auto');
        }
      });
    });

    // Control de velocidad
    const speedSlider = document.getElementById('speed-slider-dfs') as HTMLInputElement;
    const speedValue = document.getElementById('speed-value-dfs');

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
