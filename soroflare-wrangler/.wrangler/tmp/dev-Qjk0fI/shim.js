// .wrangler/tmp/bundle-Ydq7Dk/checked-fetch.js
var urls = /* @__PURE__ */ new Set();
function checkURL(request, init) {
  const url = request instanceof URL ? request : new URL(
    (typeof request === "string" ? new Request(request, init) : request).url
  );
  if (url.port && url.port !== "443" && url.protocol === "https:") {
    if (!urls.has(url.toString())) {
      urls.add(url.toString());
      console.warn(
        `WARNING: known issue with \`fetch()\` requests to custom HTTPS ports in published Workers:
 - ${url.toString()} - the custom port will be ignored when the Worker is published using the \`wrangler deploy\` command.
`
      );
    }
  }
}
globalThis.fetch = new Proxy(globalThis.fetch, {
  apply(target, thisArg, argArray) {
    const [request, init] = argArray;
    checkURL(request, init);
    return Reflect.apply(target, thisArg, argArray);
  }
});

// ../node_modules/wrangler/templates/middleware/common.ts
var __facade_middleware__ = [];
function __facade_register__(...args) {
  __facade_middleware__.push(...args.flat());
}
function __facade_invokeChain__(request, env, ctx, dispatch, middlewareChain) {
  const [head, ...tail] = middlewareChain;
  const middlewareCtx = {
    dispatch,
    next(newRequest, newEnv) {
      return __facade_invokeChain__(newRequest, newEnv, ctx, dispatch, tail);
    }
  };
  return head(request, env, ctx, middlewareCtx);
}
function __facade_invoke__(request, env, ctx, dispatch, finalMiddleware) {
  return __facade_invokeChain__(request, env, ctx, dispatch, [
    ...__facade_middleware__,
    finalMiddleware
  ]);
}

// build/worker/shim.mjs
import I from "./8cb25b654fdc834f66dc4d3140b6b4b40b10a8d0-index.wasm";
import qe from "./8cb25b654fdc834f66dc4d3140b6b4b40b10a8d0-index.wasm";
var D = Object.defineProperty;
var z = (e, t) => {
  for (var n in t)
    D(e, n, { get: t[n], enumerable: true });
};
var d = {};
z(d, { IntoUnderlyingByteSource: () => E, IntoUnderlyingSink: () => O, IntoUnderlyingSource: () => M, MinifyConfig: () => q, PipeOptions: () => S, PolishConfig: () => Y, QueuingStrategy: () => C, R2Range: () => F, ReadableStreamGetReaderOptions: () => L, RequestRedirect: () => Z, __wbg_arrayBuffer_9bd7880d4af55eb0: () => It, __wbg_buffer_610b70c8fd30da2d: () => Tt, __wbg_buffer_cf65c07de34b9a08: () => ae, __wbg_byobRequest_a3c74c3694777d1b: () => xt, __wbg_byteLength_1fef7842ca4200fa: () => Et, __wbg_byteOffset_ede786cfcf88d3dd: () => At, __wbg_bytesliteral_efe7d360639bf32b: () => Ot, __wbg_call_9495de66fdbe016b: () => ee, __wbg_call_95d1ea488d03e4e8: () => Pt, __wbg_cf_23036f27554431ca: () => Ct, __wbg_close_045ed342139beb7d: () => jt, __wbg_close_a41954830b65c455: () => yt, __wbg_constructor_0c9828c8a7cf1dc6: () => ce, __wbg_crypto_58f13aa23ffcb166: () => _t, __wbg_enqueue_3a8a8e67e44d2567: () => mt, __wbg_error_f851667af71bcfc6: () => ot, __wbg_getRandomValues_504510b5564925af: () => dt, __wbg_getTime_7c59072d1651a3cf: () => re, __wbg_get_baf4855f9a986186: () => Bt, __wbg_globalThis_87cbb8506fecf3a9: () => Yt, __wbg_global_c85a9259e621f3db: () => Zt, __wbg_headers_ab5251d2727ac41e: () => zt, __wbg_instanceof_Error_749a7378f4439ee0: () => Gt, __wbg_latitude_0037b9b738eb67a8: () => Mt, __wbg_length_27a2afe8ab42b09f: () => de, __wbg_log_7bb108d119bafbc1: () => Ft, __wbg_longitude_f0762327827bf745: () => qt, __wbg_method_d1ee174c753ca2be: () => $t, __wbg_msCrypto_abcb1295e768d1f2: () => at, __wbg_name_4e66d4cfa3e9270a: () => ne, __wbg_new0_25059e40b1c02766: () => oe, __wbg_new_15d3966e9981a196: () => Qt, __wbg_new_537b7341ce90bb31: () => pe, __wbg_new_9d3a9ce4282a18a8: () => se, __wbg_new_abda76e883ba8a5f: () => nt, __wbg_new_f1c3a9c2533a55b8: () => Nt, __wbg_new_f9876326328f45ed: () => Jt, __wbg_newnoargs_2b8b6bd7753c76ba: () => Vt, __wbg_newwithbyteoffsetandlength_9fb2f11355ecadf5: () => be, __wbg_newwithheaders_1663eaa35210e3fd: () => Ht, __wbg_newwithlength_b56c882b57805732: () => le, __wbg_newwithoptbuffersourceandinit_4d2fa6d435ff2a63: () => Lt, __wbg_newwithoptreadablestreamandinit_a0b4dc209bd176be: () => Wt, __wbg_newwithoptstrandinit_1a4621d99c54e7c3: () => Rt, __wbg_node_523d7bd03ef69fba: () => ut, __wbg_process_5b786e71d465a513: () => st, __wbg_randomFillSync_a0d98aa11c81fe89: () => gt, __wbg_region_e7b8f18d2319ee0a: () => St, __wbg_require_2784e593a4674877: () => bt, __wbg_resolve_fd40f858d9db1a04: () => ie, __wbg_respond_f4778bef04e912a6: () => vt, __wbg_self_e7c1f827057f6584: () => Kt, __wbg_set_17499e8aa4003ebd: () => ge, __wbg_set_6aa458a4ebdb65cb: () => he, __wbg_set_a5d34c36a1a4ebd1: () => Ut, __wbg_stack_658279fe44541cf6: () => rt, __wbg_subarray_7526649b91a252a6: () => we, __wbg_then_ec5db6d509eb475f: () => ue, __wbg_then_f753623316e2873a: () => fe, __wbg_toString_4f53179351070600: () => _e, __wbg_toString_cec163b212643722: () => te, __wbg_url_bd2775644ef804ec: () => Dt, __wbg_versions_c2ab80650590b6a2: () => it, __wbg_view_d1a31268af734e5d: () => kt, __wbg_window_a09ec664e14b1b81: () => Xt, __wbindgen_cb_drop: () => tt, __wbindgen_closure_wrapper4675: () => je, __wbindgen_debug_string: () => ye, __wbindgen_is_function: () => pt, __wbindgen_is_object: () => ct, __wbindgen_is_string: () => ft, __wbindgen_is_undefined: () => lt, __wbindgen_memory: () => xe, __wbindgen_number_new: () => wt, __wbindgen_object_clone_ref: () => ht, __wbindgen_object_drop_ref: () => G, __wbindgen_string_get: () => et, __wbindgen_string_new: () => Q, __wbindgen_throw: () => me, fetch: () => R, getMemory: () => H });
function W() {
  return "bytes";
}
var N = new WebAssembly.Instance(I, { "./index_bg.js": d });
var o = N.exports;
function H() {
  return o.memory;
}
var w = new Array(128).fill(void 0);
w.push(void 0, null, true, false);
function r(e) {
  return w[e];
}
var x = w.length;
function U(e) {
  e < 132 || (w[e] = x, x = e);
}
function g(e) {
  let t = r(e);
  return U(e), t;
}
var V = typeof TextDecoder > "u" ? (0, module.require)("util").TextDecoder : TextDecoder;
var $ = new V("utf-8", { ignoreBOM: true, fatal: true });
$.decode();
var j = null;
function v() {
  return (j === null || j.byteLength === 0) && (j = new Uint8Array(o.memory.buffer)), j;
}
function h(e, t) {
  return $.decode(v().subarray(e, e + t));
}
function c(e) {
  x === w.length && w.push(w.length + 1);
  let t = x;
  return x = w[t], w[t] = e, t;
}
var l = 0;
var B = typeof TextEncoder > "u" ? (0, module.require)("util").TextEncoder : TextEncoder;
var T = new B("utf-8");
var P = typeof T.encodeInto == "function" ? function(e, t) {
  return T.encodeInto(e, t);
} : function(e, t) {
  let n = T.encode(e);
  return t.set(n), { read: e.length, written: n.length };
};
function y(e, t, n) {
  if (n === void 0) {
    let a = T.encode(e), m = t(a.length);
    return v().subarray(m, m + a.length).set(a), l = a.length, m;
  }
  let _ = e.length, s = t(_), b = v(), u = 0;
  for (; u < _; u++) {
    let a = e.charCodeAt(u);
    if (a > 127)
      break;
    b[s + u] = a;
  }
  if (u !== _) {
    u !== 0 && (e = e.slice(u)), s = n(s, _, _ = u + e.length * 3);
    let a = v().subarray(s + u, s + _), m = P(e, a);
    u += m.written;
  }
  return l = u, s;
}
function p(e) {
  return e == null;
}
var k = null;
function i() {
  return (k === null || k.byteLength === 0) && (k = new Int32Array(o.memory.buffer)), k;
}
function A(e) {
  let t = typeof e;
  if (t == "number" || t == "boolean" || e == null)
    return `${e}`;
  if (t == "string")
    return `"${e}"`;
  if (t == "symbol") {
    let s = e.description;
    return s == null ? "Symbol" : `Symbol(${s})`;
  }
  if (t == "function") {
    let s = e.name;
    return typeof s == "string" && s.length > 0 ? `Function(${s})` : "Function";
  }
  if (Array.isArray(e)) {
    let s = e.length, b = "[";
    s > 0 && (b += A(e[0]));
    for (let u = 1; u < s; u++)
      b += ", " + A(e[u]);
    return b += "]", b;
  }
  let n = /\[object ([^\]]+)\]/.exec(toString.call(e)), _;
  if (n.length > 1)
    _ = n[1];
  else
    return toString.call(e);
  if (_ == "Object")
    try {
      return "Object(" + JSON.stringify(e) + ")";
    } catch {
      return "Object";
    }
  return e instanceof Error ? `${e.name}: ${e.message}
${e.stack}` : _;
}
function J(e, t, n, _) {
  let s = { a: e, b: t, cnt: 1, dtor: n }, b = (...u) => {
    s.cnt++;
    let a = s.a;
    s.a = 0;
    try {
      return _(a, s.b, ...u);
    } finally {
      --s.cnt === 0 ? o.__wbindgen_export_2.get(s.dtor)(a, s.b) : s.a = a;
    }
  };
  return b.original = s, b;
}
function K(e, t, n) {
  o._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha16e396a8a1fe0a8(e, t, c(n));
}
function R(e, t, n) {
  let _ = o.fetch(c(e), c(t), c(n));
  return g(_);
}
function f(e, t) {
  try {
    return e.apply(this, t);
  } catch (n) {
    o.__wbindgen_exn_store(c(n));
  }
}
function X(e, t, n, _) {
  o.wasm_bindgen__convert__closures__invoke2_mut__h1ef7d1f7e658d069(e, t, c(n), c(_));
}
var Y = Object.freeze({ Off: 0, 0: "Off", Lossy: 1, 1: "Lossy", Lossless: 2, 2: "Lossless" });
var Z = Object.freeze({ Error: 0, 0: "Error", Follow: 1, 1: "Follow", Manual: 2, 2: "Manual" });
var E = class {
  __destroy_into_raw() {
    let t = this.ptr;
    return this.ptr = 0, t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_intounderlyingbytesource_free(t);
  }
  get type() {
    let t = o.intounderlyingbytesource_type(this.ptr);
    return g(t);
  }
  get autoAllocateChunkSize() {
    return o.intounderlyingbytesource_autoAllocateChunkSize(this.ptr) >>> 0;
  }
  start(t) {
    o.intounderlyingbytesource_start(this.ptr, c(t));
  }
  pull(t) {
    let n = o.intounderlyingbytesource_pull(this.ptr, c(t));
    return g(n);
  }
  cancel() {
    let t = this.__destroy_into_raw();
    o.intounderlyingbytesource_cancel(t);
  }
};
var O = class {
  __destroy_into_raw() {
    let t = this.ptr;
    return this.ptr = 0, t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_intounderlyingsink_free(t);
  }
  write(t) {
    let n = o.intounderlyingsink_write(this.ptr, c(t));
    return g(n);
  }
  close() {
    let t = this.__destroy_into_raw(), n = o.intounderlyingsink_close(t);
    return g(n);
  }
  abort(t) {
    let n = this.__destroy_into_raw(), _ = o.intounderlyingsink_abort(n, c(t));
    return g(_);
  }
};
var M = class {
  __destroy_into_raw() {
    let t = this.ptr;
    return this.ptr = 0, t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_intounderlyingsource_free(t);
  }
  pull(t) {
    let n = o.intounderlyingsource_pull(this.ptr, c(t));
    return g(n);
  }
  cancel() {
    let t = this.__destroy_into_raw();
    o.intounderlyingsource_cancel(t);
  }
};
var q = class {
  __destroy_into_raw() {
    let t = this.ptr;
    return this.ptr = 0, t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_minifyconfig_free(t);
  }
  get js() {
    return o.__wbg_get_minifyconfig_js(this.ptr) !== 0;
  }
  set js(t) {
    o.__wbg_set_minifyconfig_js(this.ptr, t);
  }
  get html() {
    return o.__wbg_get_minifyconfig_html(this.ptr) !== 0;
  }
  set html(t) {
    o.__wbg_set_minifyconfig_html(this.ptr, t);
  }
  get css() {
    return o.__wbg_get_minifyconfig_css(this.ptr) !== 0;
  }
  set css(t) {
    o.__wbg_set_minifyconfig_css(this.ptr, t);
  }
};
var S = class {
  __destroy_into_raw() {
    let t = this.ptr;
    return this.ptr = 0, t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_pipeoptions_free(t);
  }
  get preventClose() {
    return o.pipeoptions_preventClose(this.ptr) !== 0;
  }
  get preventCancel() {
    return o.pipeoptions_preventCancel(this.ptr) !== 0;
  }
  get preventAbort() {
    return o.pipeoptions_preventAbort(this.ptr) !== 0;
  }
  get signal() {
    let t = o.pipeoptions_signal(this.ptr);
    return g(t);
  }
};
var C = class {
  __destroy_into_raw() {
    let t = this.ptr;
    return this.ptr = 0, t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_queuingstrategy_free(t);
  }
  get highWaterMark() {
    return o.queuingstrategy_highWaterMark(this.ptr);
  }
};
var F = class {
  __destroy_into_raw() {
    let t = this.ptr;
    return this.ptr = 0, t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_r2range_free(t);
  }
  get offset() {
    try {
      let _ = o.__wbindgen_add_to_stack_pointer(-16);
      o.__wbg_get_r2range_offset(_, this.ptr);
      var t = i()[_ / 4 + 0], n = i()[_ / 4 + 1];
      return t === 0 ? void 0 : n >>> 0;
    } finally {
      o.__wbindgen_add_to_stack_pointer(16);
    }
  }
  set offset(t) {
    o.__wbg_set_r2range_offset(this.ptr, !p(t), p(t) ? 0 : t);
  }
  get length() {
    try {
      let _ = o.__wbindgen_add_to_stack_pointer(-16);
      o.__wbg_get_r2range_length(_, this.ptr);
      var t = i()[_ / 4 + 0], n = i()[_ / 4 + 1];
      return t === 0 ? void 0 : n >>> 0;
    } finally {
      o.__wbindgen_add_to_stack_pointer(16);
    }
  }
  set length(t) {
    o.__wbg_set_r2range_length(this.ptr, !p(t), p(t) ? 0 : t);
  }
  get suffix() {
    try {
      let _ = o.__wbindgen_add_to_stack_pointer(-16);
      o.__wbg_get_r2range_suffix(_, this.ptr);
      var t = i()[_ / 4 + 0], n = i()[_ / 4 + 1];
      return t === 0 ? void 0 : n >>> 0;
    } finally {
      o.__wbindgen_add_to_stack_pointer(16);
    }
  }
  set suffix(t) {
    o.__wbg_set_r2range_suffix(this.ptr, !p(t), p(t) ? 0 : t);
  }
};
var L = class {
  __destroy_into_raw() {
    let t = this.ptr;
    return this.ptr = 0, t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_readablestreamgetreaderoptions_free(t);
  }
  get mode() {
    let t = o.readablestreamgetreaderoptions_mode(this.ptr);
    return g(t);
  }
};
function G(e) {
  g(e);
}
function Q(e, t) {
  let n = h(e, t);
  return c(n);
}
function tt(e) {
  let t = g(e).original;
  return t.cnt-- == 1 ? (t.a = 0, true) : false;
}
function et(e, t) {
  let n = r(t), _ = typeof n == "string" ? n : void 0;
  var s = p(_) ? 0 : y(_, o.__wbindgen_malloc, o.__wbindgen_realloc), b = l;
  i()[e / 4 + 1] = b, i()[e / 4 + 0] = s;
}
function nt() {
  let e = new Error();
  return c(e);
}
function rt(e, t) {
  let n = r(t).stack, _ = y(n, o.__wbindgen_malloc, o.__wbindgen_realloc), s = l;
  i()[e / 4 + 1] = s, i()[e / 4 + 0] = _;
}
function ot(e, t) {
  try {
    console.error(h(e, t));
  } finally {
    o.__wbindgen_free(e, t);
  }
}
function _t(e) {
  let t = r(e).crypto;
  return c(t);
}
function ct(e) {
  let t = r(e);
  return typeof t == "object" && t !== null;
}
function st(e) {
  let t = r(e).process;
  return c(t);
}
function it(e) {
  let t = r(e).versions;
  return c(t);
}
function ut(e) {
  let t = r(e).node;
  return c(t);
}
function ft(e) {
  return typeof r(e) == "string";
}
function at(e) {
  let t = r(e).msCrypto;
  return c(t);
}
function bt() {
  return f(function() {
    let e = module.require;
    return c(e);
  }, arguments);
}
function pt(e) {
  return typeof r(e) == "function";
}
function gt() {
  return f(function(e, t) {
    r(e).randomFillSync(g(t));
  }, arguments);
}
function dt() {
  return f(function(e, t) {
    r(e).getRandomValues(r(t));
  }, arguments);
}
function lt(e) {
  return r(e) === void 0;
}
function wt(e) {
  return c(e);
}
function ht(e) {
  let t = r(e);
  return c(t);
}
function yt(e) {
  r(e).close();
}
function mt(e, t) {
  r(e).enqueue(r(t));
}
function xt(e) {
  let t = r(e).byobRequest;
  return p(t) ? 0 : c(t);
}
function jt(e) {
  r(e).close();
}
function kt(e) {
  let t = r(e).view;
  return p(t) ? 0 : c(t);
}
function vt(e, t) {
  r(e).respond(t >>> 0);
}
function Tt(e) {
  let t = r(e).buffer;
  return c(t);
}
function At(e) {
  return r(e).byteOffset;
}
function Et(e) {
  return r(e).byteLength;
}
function Ot() {
  let e = W();
  return c(e);
}
function Mt(e, t) {
  let n = r(t).latitude;
  var _ = p(n) ? 0 : y(n, o.__wbindgen_malloc, o.__wbindgen_realloc), s = l;
  i()[e / 4 + 1] = s, i()[e / 4 + 0] = _;
}
function qt(e, t) {
  let n = r(t).longitude;
  var _ = p(n) ? 0 : y(n, o.__wbindgen_malloc, o.__wbindgen_realloc), s = l;
  i()[e / 4 + 1] = s, i()[e / 4 + 0] = _;
}
function St(e, t) {
  let n = r(t).region;
  var _ = p(n) ? 0 : y(n, o.__wbindgen_malloc, o.__wbindgen_realloc), s = l;
  i()[e / 4 + 1] = s, i()[e / 4 + 0] = _;
}
function Ct(e) {
  let t = r(e).cf;
  return p(t) ? 0 : c(t);
}
function Ft(e) {
  console.log(r(e));
}
function Lt() {
  return f(function(e, t) {
    let n = new Response(r(e), r(t));
    return c(n);
  }, arguments);
}
function Rt() {
  return f(function(e, t, n) {
    let _ = new Response(e === 0 ? void 0 : h(e, t), r(n));
    return c(_);
  }, arguments);
}
function Wt() {
  return f(function(e, t) {
    let n = new Response(r(e), r(t));
    return c(n);
  }, arguments);
}
function $t(e, t) {
  let n = r(t).method, _ = y(n, o.__wbindgen_malloc, o.__wbindgen_realloc), s = l;
  i()[e / 4 + 1] = s, i()[e / 4 + 0] = _;
}
function Dt(e, t) {
  let n = r(t).url, _ = y(n, o.__wbindgen_malloc, o.__wbindgen_realloc), s = l;
  i()[e / 4 + 1] = s, i()[e / 4 + 0] = _;
}
function zt(e) {
  let t = r(e).headers;
  return c(t);
}
function It() {
  return f(function(e) {
    let t = r(e).arrayBuffer();
    return c(t);
  }, arguments);
}
function Nt() {
  return f(function() {
    let e = new Headers();
    return c(e);
  }, arguments);
}
function Ht() {
  return f(function(e) {
    let t = new Headers(r(e));
    return c(t);
  }, arguments);
}
function Ut() {
  return f(function(e, t, n, _, s) {
    r(e).set(h(t, n), h(_, s));
  }, arguments);
}
function Vt(e, t) {
  let n = new Function(h(e, t));
  return c(n);
}
function Bt() {
  return f(function(e, t) {
    let n = Reflect.get(r(e), r(t));
    return c(n);
  }, arguments);
}
function Pt() {
  return f(function(e, t) {
    let n = r(e).call(r(t));
    return c(n);
  }, arguments);
}
function Jt() {
  let e = new Object();
  return c(e);
}
function Kt() {
  return f(function() {
    let e = self.self;
    return c(e);
  }, arguments);
}
function Xt() {
  return f(function() {
    let e = window.window;
    return c(e);
  }, arguments);
}
function Yt() {
  return f(function() {
    let e = globalThis.globalThis;
    return c(e);
  }, arguments);
}
function Zt() {
  return f(function() {
    let e = global.global;
    return c(e);
  }, arguments);
}
function Gt(e) {
  let t;
  try {
    t = r(e) instanceof Error;
  } catch {
    t = false;
  }
  return t;
}
function Qt(e, t) {
  let n = new Error(h(e, t));
  return c(n);
}
function te(e) {
  let t = r(e).toString();
  return c(t);
}
function ee() {
  return f(function(e, t, n) {
    let _ = r(e).call(r(t), r(n));
    return c(_);
  }, arguments);
}
function ne(e) {
  let t = r(e).name;
  return c(t);
}
function re(e) {
  return r(e).getTime();
}
function oe() {
  return c(/* @__PURE__ */ new Date());
}
function _e(e) {
  let t = r(e).toString();
  return c(t);
}
function ce(e) {
  let t = r(e).constructor;
  return c(t);
}
function se(e, t) {
  try {
    var n = { a: e, b: t }, _ = (b, u) => {
      let a = n.a;
      n.a = 0;
      try {
        return X(a, n.b, b, u);
      } finally {
        n.a = a;
      }
    };
    let s = new Promise(_);
    return c(s);
  } finally {
    n.a = n.b = 0;
  }
}
function ie(e) {
  let t = Promise.resolve(r(e));
  return c(t);
}
function ue(e, t) {
  let n = r(e).then(r(t));
  return c(n);
}
function fe(e, t, n) {
  let _ = r(e).then(r(t), r(n));
  return c(_);
}
function ae(e) {
  let t = r(e).buffer;
  return c(t);
}
function be(e, t, n) {
  let _ = new Uint8Array(r(e), t >>> 0, n >>> 0);
  return c(_);
}
function pe(e) {
  let t = new Uint8Array(r(e));
  return c(t);
}
function ge(e, t, n) {
  r(e).set(r(t), n >>> 0);
}
function de(e) {
  return r(e).length;
}
function le(e) {
  let t = new Uint8Array(e >>> 0);
  return c(t);
}
function we(e, t, n) {
  let _ = r(e).subarray(t >>> 0, n >>> 0);
  return c(_);
}
function he() {
  return f(function(e, t, n) {
    return Reflect.set(r(e), r(t), r(n));
  }, arguments);
}
function ye(e, t) {
  let n = A(r(t)), _ = y(n, o.__wbindgen_malloc, o.__wbindgen_realloc), s = l;
  i()[e / 4 + 1] = s, i()[e / 4 + 0] = _;
}
function me(e, t) {
  throw new Error(h(e, t));
}
function xe() {
  let e = o.memory;
  return c(e);
}
function je(e, t, n) {
  let _ = J(e, t, 1097, K);
  return c(_);
}
var Se = { fetch: R, scheduled: void 0, queue: void 0 };

// ../node_modules/wrangler/templates/middleware/middleware-miniflare3-json-error.ts
function reduceError(e) {
  return {
    name: e?.name,
    message: e?.message ?? String(e),
    stack: e?.stack,
    cause: e?.cause === void 0 ? void 0 : reduceError(e.cause)
  };
}
var jsonError = async (request, env, _ctx, middlewareCtx) => {
  try {
    return await middlewareCtx.next(request, env);
  } catch (e) {
    const error = reduceError(e);
    return Response.json(error, {
      status: 500,
      headers: { "MF-Experimental-Error-Stack": "true" }
    });
  }
};
var middleware_miniflare3_json_error_default = jsonError;
var wrap = void 0;

// .wrangler/tmp/bundle-Ydq7Dk/middleware-insertion-facade.js
var envWrappers = [wrap].filter(Boolean);
var facade = {
  ...Se,
  envWrappers,
  middleware: [
    middleware_miniflare3_json_error_default,
    ...Se.middleware ? Se.middleware : []
  ].filter(Boolean)
};
var middleware_insertion_facade_default = facade;

// .wrangler/tmp/bundle-Ydq7Dk/middleware-loader.entry.ts
var __Facade_ScheduledController__ = class {
  constructor(scheduledTime, cron, noRetry) {
    this.scheduledTime = scheduledTime;
    this.cron = cron;
    this.#noRetry = noRetry;
  }
  #noRetry;
  noRetry() {
    if (!(this instanceof __Facade_ScheduledController__)) {
      throw new TypeError("Illegal invocation");
    }
    this.#noRetry();
  }
};
var __facade_modules_fetch__ = function(request, env, ctx) {
  if (middleware_insertion_facade_default.fetch === void 0)
    throw new Error("Handler does not export a fetch() function.");
  return middleware_insertion_facade_default.fetch(request, env, ctx);
};
function getMaskedEnv(rawEnv) {
  let env = rawEnv;
  if (middleware_insertion_facade_default.envWrappers && middleware_insertion_facade_default.envWrappers.length > 0) {
    for (const wrapFn of middleware_insertion_facade_default.envWrappers) {
      env = wrapFn(env);
    }
  }
  return env;
}
var registeredMiddleware = false;
var facade2 = {
  ...middleware_insertion_facade_default.tail && {
    tail: maskHandlerEnv(middleware_insertion_facade_default.tail)
  },
  ...middleware_insertion_facade_default.trace && {
    trace: maskHandlerEnv(middleware_insertion_facade_default.trace)
  },
  ...middleware_insertion_facade_default.scheduled && {
    scheduled: maskHandlerEnv(middleware_insertion_facade_default.scheduled)
  },
  ...middleware_insertion_facade_default.queue && {
    queue: maskHandlerEnv(middleware_insertion_facade_default.queue)
  },
  ...middleware_insertion_facade_default.test && {
    test: maskHandlerEnv(middleware_insertion_facade_default.test)
  },
  ...middleware_insertion_facade_default.email && {
    email: maskHandlerEnv(middleware_insertion_facade_default.email)
  },
  fetch(request, rawEnv, ctx) {
    const env = getMaskedEnv(rawEnv);
    if (middleware_insertion_facade_default.middleware && middleware_insertion_facade_default.middleware.length > 0) {
      if (!registeredMiddleware) {
        registeredMiddleware = true;
        for (const middleware of middleware_insertion_facade_default.middleware) {
          __facade_register__(middleware);
        }
      }
      const __facade_modules_dispatch__ = function(type, init) {
        if (type === "scheduled" && middleware_insertion_facade_default.scheduled !== void 0) {
          const controller = new __Facade_ScheduledController__(
            Date.now(),
            init.cron ?? "",
            () => {
            }
          );
          return middleware_insertion_facade_default.scheduled(controller, env, ctx);
        }
      };
      return __facade_invoke__(
        request,
        env,
        ctx,
        __facade_modules_dispatch__,
        __facade_modules_fetch__
      );
    } else {
      return __facade_modules_fetch__(request, env, ctx);
    }
  }
};
function maskHandlerEnv(handler) {
  return (data, env, ctx) => handler(data, getMaskedEnv(env), ctx);
}
var middleware_loader_entry_default = facade2;
export {
  E as IntoUnderlyingByteSource,
  O as IntoUnderlyingSink,
  M as IntoUnderlyingSource,
  q as MinifyConfig,
  S as PipeOptions,
  Y as PolishConfig,
  C as QueuingStrategy,
  F as R2Range,
  L as ReadableStreamGetReaderOptions,
  Z as RequestRedirect,
  It as __wbg_arrayBuffer_9bd7880d4af55eb0,
  Tt as __wbg_buffer_610b70c8fd30da2d,
  ae as __wbg_buffer_cf65c07de34b9a08,
  xt as __wbg_byobRequest_a3c74c3694777d1b,
  Et as __wbg_byteLength_1fef7842ca4200fa,
  At as __wbg_byteOffset_ede786cfcf88d3dd,
  Ot as __wbg_bytesliteral_efe7d360639bf32b,
  ee as __wbg_call_9495de66fdbe016b,
  Pt as __wbg_call_95d1ea488d03e4e8,
  Ct as __wbg_cf_23036f27554431ca,
  jt as __wbg_close_045ed342139beb7d,
  yt as __wbg_close_a41954830b65c455,
  ce as __wbg_constructor_0c9828c8a7cf1dc6,
  _t as __wbg_crypto_58f13aa23ffcb166,
  mt as __wbg_enqueue_3a8a8e67e44d2567,
  ot as __wbg_error_f851667af71bcfc6,
  dt as __wbg_getRandomValues_504510b5564925af,
  re as __wbg_getTime_7c59072d1651a3cf,
  Bt as __wbg_get_baf4855f9a986186,
  Yt as __wbg_globalThis_87cbb8506fecf3a9,
  Zt as __wbg_global_c85a9259e621f3db,
  zt as __wbg_headers_ab5251d2727ac41e,
  Gt as __wbg_instanceof_Error_749a7378f4439ee0,
  Mt as __wbg_latitude_0037b9b738eb67a8,
  de as __wbg_length_27a2afe8ab42b09f,
  Ft as __wbg_log_7bb108d119bafbc1,
  qt as __wbg_longitude_f0762327827bf745,
  $t as __wbg_method_d1ee174c753ca2be,
  at as __wbg_msCrypto_abcb1295e768d1f2,
  ne as __wbg_name_4e66d4cfa3e9270a,
  oe as __wbg_new0_25059e40b1c02766,
  Qt as __wbg_new_15d3966e9981a196,
  pe as __wbg_new_537b7341ce90bb31,
  se as __wbg_new_9d3a9ce4282a18a8,
  nt as __wbg_new_abda76e883ba8a5f,
  Nt as __wbg_new_f1c3a9c2533a55b8,
  Jt as __wbg_new_f9876326328f45ed,
  Vt as __wbg_newnoargs_2b8b6bd7753c76ba,
  be as __wbg_newwithbyteoffsetandlength_9fb2f11355ecadf5,
  Ht as __wbg_newwithheaders_1663eaa35210e3fd,
  le as __wbg_newwithlength_b56c882b57805732,
  Lt as __wbg_newwithoptbuffersourceandinit_4d2fa6d435ff2a63,
  Wt as __wbg_newwithoptreadablestreamandinit_a0b4dc209bd176be,
  Rt as __wbg_newwithoptstrandinit_1a4621d99c54e7c3,
  ut as __wbg_node_523d7bd03ef69fba,
  st as __wbg_process_5b786e71d465a513,
  gt as __wbg_randomFillSync_a0d98aa11c81fe89,
  St as __wbg_region_e7b8f18d2319ee0a,
  bt as __wbg_require_2784e593a4674877,
  ie as __wbg_resolve_fd40f858d9db1a04,
  vt as __wbg_respond_f4778bef04e912a6,
  Kt as __wbg_self_e7c1f827057f6584,
  ge as __wbg_set_17499e8aa4003ebd,
  he as __wbg_set_6aa458a4ebdb65cb,
  Ut as __wbg_set_a5d34c36a1a4ebd1,
  rt as __wbg_stack_658279fe44541cf6,
  we as __wbg_subarray_7526649b91a252a6,
  ue as __wbg_then_ec5db6d509eb475f,
  fe as __wbg_then_f753623316e2873a,
  _e as __wbg_toString_4f53179351070600,
  te as __wbg_toString_cec163b212643722,
  Dt as __wbg_url_bd2775644ef804ec,
  it as __wbg_versions_c2ab80650590b6a2,
  kt as __wbg_view_d1a31268af734e5d,
  Xt as __wbg_window_a09ec664e14b1b81,
  tt as __wbindgen_cb_drop,
  je as __wbindgen_closure_wrapper4675,
  ye as __wbindgen_debug_string,
  pt as __wbindgen_is_function,
  ct as __wbindgen_is_object,
  ft as __wbindgen_is_string,
  lt as __wbindgen_is_undefined,
  xe as __wbindgen_memory,
  wt as __wbindgen_number_new,
  ht as __wbindgen_object_clone_ref,
  G as __wbindgen_object_drop_ref,
  et as __wbindgen_string_get,
  Q as __wbindgen_string_new,
  me as __wbindgen_throw,
  middleware_loader_entry_default as default,
  R as fetch,
  H as getMemory,
  qe as wasmModule
};
//# sourceMappingURL=shim.js.map
