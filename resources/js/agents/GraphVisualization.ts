import * as d3 from 'd3';
import { Grid } from './Grid';
import { Position } from './types';

/**
 * Nodo del grafo
 */
interface GraphNode {
  id: string;
  row: number;
  col: number;
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
  type: 'normal' | 'start' | 'goal';
}

/**
 * Arista del grafo
 */
interface GraphEdge {
  source: string | GraphNode;
  target: string | GraphNode;
  action: string;
}

/**
 * Componente de visualización del grafo de estados
 */
export class GraphVisualization {
  private container: HTMLElement;
  private grid: Grid;
  private svg: d3.Selection<SVGSVGElement, unknown, null, undefined> | null = null;
  private width: number = 1200;
  private height: number = 800;
  private simulation: d3.Simulation<GraphNode, GraphEdge> | null = null;
  private g: d3.Selection<SVGGElement, unknown, null, undefined> | null = null;
  private zoom: d3.ZoomBehavior<SVGSVGElement, unknown> | null = null;

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
   * Renderiza el componente
   */
  render(): void {
    const { nodes, edges } = this.generateGraph();
    this.renderGraph(nodes, edges);
  }

  /**
   * Genera los nodos y aristas del grafo
   */
  private generateGraph(): { nodes: GraphNode[], edges: GraphEdge[] } {
    const config = this.grid.getConfig();
    const startPos = this.grid.getStartPosition();
    const goalPositions = this.grid.getGoalPositions();
    const obstacles = this.getObstacles();
    const obstacleSet = new Set(obstacles.map(pos => `${pos.row},${pos.col}`));

    const nodes: GraphNode[] = [];
    const edges: GraphEdge[] = [];

    const goalSet = new Set(goalPositions.map(pos => `${pos.row},${pos.col}`));

    // Generar nodos
    for (let row = 0; row < config.rows; row++) {
      for (let col = 0; col < config.cols; col++) {
        if (obstacleSet.has(`${row},${col}`)) continue;

        const id = `${row},${col}`;
        let type: 'normal' | 'start' | 'goal' = 'normal';

        if (startPos && startPos.row === row && startPos.col === col) {
          type = 'start';
        } else if (goalSet.has(id)) {
          type = 'goal';
        }

        nodes.push({ id, row, col, type });
      }
    }

    // Generar aristas
    const actions = [
      { name: 'Arriba', deltaRow: -1, deltaCol: 0 },
      { name: 'Abajo', deltaRow: 1, deltaCol: 0 },
      { name: 'Izquierda', deltaRow: 0, deltaCol: -1 },
      { name: 'Derecha', deltaRow: 0, deltaCol: 1 }
    ];

    for (const node of nodes) {
      for (const action of actions) {
        const newRow = node.row + action.deltaRow;
        const newCol = node.col + action.deltaCol;

        // Verificar si la transición es válida
        const isValid =
          newRow >= 0 && newRow < config.rows &&
          newCol >= 0 && newCol < config.cols &&
          !obstacleSet.has(`${newRow},${newCol}`);

        if (isValid) {
          edges.push({
            source: node.id,
            target: `${newRow},${newCol}`,
            action: action.name
          });
        }
      }
    }

    return { nodes, edges };
  }

  /**
   * Renderiza el grafo con D3.js
   */
  private renderGraph(nodes: GraphNode[], edges: GraphEdge[]): void {
    // Limpiar el contenedor completamente
    this.container.innerHTML = '';

    // Si no hay nodos, no renderizar nada
    if (nodes.length === 0) {
      console.warn('[GraphVisualization] No nodes to render');
      return;
    }

    // Crear el contenedor SVG directamente
    const vizContainer = this.container;

    // Limpiar SVG anterior
    if (this.svg) {
      this.svg.remove();
    }

    // Obtener dimensiones del contenedor
    const containerRect = vizContainer.getBoundingClientRect();
    this.width = containerRect.width;
    this.height = containerRect.height;

    // Crear SVG
    this.svg = d3.select(vizContainer)
      .append('svg')
      .attr('width', '100%')
      .attr('height', '100%')
      .attr('viewBox', `0 0 ${this.width} ${this.height}`);

    // Crear grupo principal para zoom/pan
    this.g = this.svg.append('g');

    // Configurar zoom y pan
    this.zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        if (this.g) {
          this.g.attr('transform', event.transform);
        }
      });

    this.svg.call(this.zoom);

    // Definir marcadores para las flechas
    this.svg.append('defs').append('marker')
      .attr('id', 'arrowhead')
      .attr('viewBox', '-0 -5 10 10')
      .attr('refX', 20)
      .attr('refY', 0)
      .attr('orient', 'auto')
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .append('svg:path')
      .attr('d', 'M 0,-5 L 10,0 L 0,5')
      .attr('fill', '#6b7280');

    // Crear simulación de fuerzas con más espacio
    this.simulation = d3.forceSimulation<GraphNode>(nodes)
      .force('link', d3.forceLink<GraphNode, GraphEdge>(edges)
        .id(d => d.id)
        .distance(100))
      .force('charge', d3.forceManyBody().strength(-500))
      .force('center', d3.forceCenter(this.width / 2, this.height / 2))
      .force('collision', d3.forceCollide().radius(40))
      .force('x', d3.forceX(this.width / 2).strength(0.05))
      .force('y', d3.forceY(this.height / 2).strength(0.05));

    // Crear aristas en el grupo principal
    const link = this.g!.append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(edges)
      .enter()
      .append('line')
      .attr('stroke', '#6b7280')
      .attr('stroke-width', 1.5)
      .attr('stroke-opacity', 0.4)
      .attr('marker-end', 'url(#arrowhead)');

    // Crear nodos en el grupo principal
    const node = this.g!.append('g')
      .attr('class', 'nodes')
      .selectAll('g')
      .data(nodes)
      .enter()
      .append('g')
      .call(d3.drag<SVGGElement, GraphNode>()
        .on('start', (event, d) => this.dragstarted(event, d))
        .on('drag', (event, d) => this.dragged(event, d))
        .on('end', (event, d) => this.dragended(event, d)));

    // Círculos de los nodos
    node.append('circle')
      .attr('r', 15)
      .attr('fill', d => this.getNodeColor(d.type))
      .attr('stroke', d => this.getNodeStroke(d.type))
      .attr('stroke-width', 3)
      .style('cursor', 'grab')
      .on('mousedown', function() {
        d3.select(this).style('cursor', 'grabbing');
      })
      .on('mouseup', function() {
        d3.select(this).style('cursor', 'grab');
      });

    // Etiquetas de los nodos
    node.append('text')
      .attr('class', 'node-label')
      .attr('dx', 0)
      .attr('dy', -22)
      .attr('text-anchor', 'middle')
      .attr('font-size', '11px')
      .attr('font-weight', 'bold')
      .attr('fill', '#1f2937')
      .attr('stroke', '#ffffff')
      .attr('stroke-width', '3px')
      .attr('paint-order', 'stroke')
      .text(d => `(${d.row + 1},${d.col + 1})`)
      .style('pointer-events', 'none');

    // Tooltip
    node.append('title')
      .text(d => `Estado: (${d.row + 1}, ${d.col + 1})\nTipo: ${d.type}`);

    // Actualizar posiciones en cada tick
    this.simulation.on('tick', () => {
      link
        .attr('x1', d => (d.source as GraphNode).x!)
        .attr('y1', d => (d.source as GraphNode).y!)
        .attr('x2', d => (d.target as GraphNode).x!)
        .attr('y2', d => (d.target as GraphNode).y!);

      node.attr('transform', d => `translate(${d.x},${d.y})`);
    });
  }

  /**
   * Obtiene el color del nodo según su tipo
   */
  private getNodeColor(type: string): string {
    switch (type) {
      case 'start': return '#10B981'; // success token
      case 'goal': return '#EF4444';  // danger token
      default: return '#8B5CF6';      // purple-blue-500
    }
  }

  /**
   * Obtiene el color del borde del nodo según su tipo
   */
  private getNodeStroke(type: string): string {
    switch (type) {
      case 'start': return '#059669'; // green-600 (darker success)
      case 'goal': return '#DC2626';  // red-600 (darker danger)
      default: return '#7C3AED';      // purple-blue-600
    }
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
   * Manejadores de arrastre
   */
  private dragstarted(event: any, d: GraphNode): void {
    if (!event.active && this.simulation) this.simulation.alphaTarget(0.3).restart();
    d.fx = d.x;
    d.fy = d.y;
  }

  private dragged(event: any, d: GraphNode): void {
    d.fx = event.x;
    d.fy = event.y;
  }

  private dragended(event: any, d: GraphNode): void {
    if (!event.active && this.simulation) this.simulation.alphaTarget(0);
    d.fx = null;
    d.fy = null;
  }

  /**
   * Reinicia las posiciones de los nodos
   */
  resetPositions(): void {
    if (this.simulation) {
      this.simulation.alpha(1).restart();
    }
  }

  /**
   * Alterna la visibilidad de las etiquetas
   */
  toggleLabels(): void {
    if (this.g) {
      const labels = this.g.selectAll('.node-label');
      const isVisible = labels.style('display') !== 'none';
      labels.style('display', isVisible ? 'none' : 'block');
    }
  }

  /**
   * Aumenta el zoom
   */
  zoomIn(): void {
    if (this.svg && this.zoom) {
      this.svg.transition().duration(300).call(this.zoom.scaleBy, 1.3);
    }
  }

  /**
   * Reduce el zoom
   */
  zoomOut(): void {
    if (this.svg && this.zoom) {
      this.svg.transition().duration(300).call(this.zoom.scaleBy, 0.7);
    }
  }

  /**
   * Restablece el zoom
   */
  resetZoom(): void {
    if (this.svg && this.zoom) {
      this.svg.transition().duration(500).call(
        this.zoom.transform,
        d3.zoomIdentity.translate(0, 0).scale(1)
      );
    }
  }

  /**
   * Actualiza la visualización y retorna estadísticas
   */
  update(): { nodes: number; edges: number; avgDegree: string } {
    if (this.simulation) {
      this.simulation.stop();
    }

    const { nodes, edges } = this.generateGraph();

    console.log(`[GraphVisualization] Updating with ${nodes.length} nodes and ${edges.length} edges`);

    this.renderGraph(nodes, edges);

    return {
      nodes: nodes.length,
      edges: edges.length,
      avgDegree: nodes.length > 0 ? (edges.length / nodes.length).toFixed(2) : '0'
    };
  }
}
