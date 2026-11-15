import { BANDS, WAVE } from "../../../lib/stores/visual.svelte";
import { FRAGMENTS, VERTEX, type Style } from "./shaders";

export interface Scene {
  style: Style;
  time: number;
  center: [number, number];
  cover: [number, number];
  colors: Float32Array;
  energy: Float32Array;
  bands: Uint8Array;
  wave: Uint8Array;
  scale: number;
}

const UNIFORMS = ["u_size", "u_time", "u_center", "u_cover", "u_colors", "u_energy", "u_scale", "u_bands", "u_wave"] as const;

interface Program {
  program: WebGLProgram;
  uniforms: Record<(typeof UNIFORMS)[number], WebGLUniformLocation | null>;
}

/** Draws one full-screen shader per style, fed the bands and waveform as textures. */
export class Renderer {
  #canvas: HTMLCanvasElement;
  #gl: WebGL2RenderingContext;
  #programs = new Map<Style, Program>();
  #bands: WebGLTexture;
  #wave: WebGLTexture;

  constructor(canvas: HTMLCanvasElement) {
    this.#canvas = canvas;
    const gl = canvas.getContext("webgl2", { alpha: true, premultipliedAlpha: true, antialias: false, depth: false, stencil: false, powerPreference: "low-power" })!;
    gl.bindVertexArray(gl.createVertexArray());
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    this.#gl = gl;
    this.#bands = this.#texture(0, BANDS);
    this.#wave = this.#texture(1, WAVE);
  }

  resize(width: number, height: number) {
    this.#canvas.width = width;
    this.#canvas.height = height;
    this.#gl.viewport(0, 0, width, height);
  }

  draw(scene: Scene) {
    const gl = this.#gl;
    const { program, uniforms } = this.#program(scene.style);
    gl.useProgram(program);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.#bands);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, BANDS, 1, gl.RED, gl.UNSIGNED_BYTE, scene.bands);
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.#wave);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, WAVE, 1, gl.RED, gl.UNSIGNED_BYTE, scene.wave);
    gl.uniform2f(uniforms.u_size, this.#canvas.width, this.#canvas.height);
    gl.uniform1f(uniforms.u_time, scene.time);
    gl.uniform2f(uniforms.u_center, ...scene.center);
    gl.uniform2f(uniforms.u_cover, ...scene.cover);
    gl.uniform3fv(uniforms.u_colors, scene.colors);
    gl.uniform3fv(uniforms.u_energy, scene.energy);
    gl.uniform1f(uniforms.u_scale, scene.scale);
    gl.uniform1i(uniforms.u_bands, 0);
    gl.uniform1i(uniforms.u_wave, 1);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }

  destroy() {
    this.#gl.getExtension("WEBGL_lose_context")?.loseContext();
  }

  #texture(unit: number, width: number): WebGLTexture {
    const gl = this.#gl;
    const texture = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, width, 1, 0, gl.RED, gl.UNSIGNED_BYTE, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    return texture;
  }

  #program(style: Style): Program {
    const cached = this.#programs.get(style);
    if (cached) return cached;
    const gl = this.#gl;
    const program = gl.createProgram();
    gl.attachShader(program, this.#shader(gl.VERTEX_SHADER, VERTEX));
    gl.attachShader(program, this.#shader(gl.FRAGMENT_SHADER, FRAGMENTS[style]));
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program) ?? "The visualizer shader didn't link");
    const uniforms = Object.fromEntries(UNIFORMS.map((name) => [name, gl.getUniformLocation(program, name)])) as Program["uniforms"];
    const built = { program, uniforms };
    this.#programs.set(style, built);
    return built;
  }

  #shader(type: GLenum, source: string): WebGLShader {
    const gl = this.#gl;
    const shader = gl.createShader(type)!;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(shader) ?? "The visualizer shader didn't compile");
    return shader;
  }
}
