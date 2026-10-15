// Synthesised sound: no files. Everything is guarded so a missing AudioContext never breaks play.
var AU = { ctx: null, master: null, noiseBuf: null };

function auStart() {
  try {
    if (!AU.ctx) {
      var C = window.AudioContext || window.webkitAudioContext;
      if (!C) return;
      AU.ctx = new C();
      AU.master = AU.ctx.createGain();
      AU.master.gain.value = 0.5;
      AU.master.connect(AU.ctx.destination);
      var len = AU.ctx.sampleRate * 1;
      AU.noiseBuf = AU.ctx.createBuffer(1, len, AU.ctx.sampleRate);
      var d = AU.noiseBuf.getChannelData(0);
      for (var i = 0; i < len; i++) d[i] = Math.random() * 2 - 1;
    }
    if (AU.ctx.state === 'suspended') AU.ctx.resume();
  } catch (e) { AU.ctx = null; }
}

function auOK() { return !!(AU.ctx && AU.master); }

// noise burst through a filter. type: lowpass|highpass|bandpass
function snd_noise(dur, type, freq, gain, q, attack, when) {
  if (!auOK()) return;
  try {
    var c = AU.ctx, t0 = c.currentTime + (when || 0);
    var s = c.createBufferSource(); s.buffer = AU.noiseBuf; s.loop = true;
    var f = c.createBiquadFilter(); f.type = type || 'lowpass'; f.frequency.value = freq || 1000; f.Q.value = q || 0.7;
    var g = c.createGain();
    var a = attack || 0.004;
    g.gain.setValueAtTime(0.0001, t0);
    g.gain.linearRampToValueAtTime(gain || 0.2, t0 + a);
    g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
    s.connect(f); f.connect(g); g.connect(AU.master);
    s.start(t0, Math.random() * 0.5); s.stop(t0 + dur + 0.05);
  } catch (e) {}
}

function snd_tone(freq, dur, type, gain, glideTo, when, attack) {
  if (!auOK()) return;
  try {
    var c = AU.ctx, t0 = c.currentTime + (when || 0);
    var o = c.createOscillator(); o.type = type || 'sine';
    o.frequency.setValueAtTime(freq, t0);
    if (glideTo) o.frequency.exponentialRampToValueAtTime(glideTo, t0 + dur);
    var g = c.createGain();
    g.gain.setValueAtTime(0.0001, t0);
    g.gain.linearRampToValueAtTime(gain || 0.15, t0 + (attack || 0.005));
    g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
    o.connect(g); g.connect(AU.master);
    o.start(t0); o.stop(t0 + dur + 0.05);
  } catch (e) {}
}

var SND = {
  tick: function (alt) { snd_tone(alt ? 1600 : 1800, 0.03, 'square', 0.015); },
  bird: function () {
    var f = 2500 + Math.random() * 1500;
    snd_tone(f, 0.08, 'sine', 0.025, f * 1.25);
    snd_tone(f * 1.1, 0.1, 'sine', 0.025, f * 1.4, 0.13);
  },
  wheee: function () { snd_tone(500, 0.9, 'sawtooth', 0.012, 900); },
  crack: function () { snd_noise(0.05, 'bandpass', 2000, 0.3, 2); snd_tone(900, 0.02, 'square', 0.1); },
  plop: function () { snd_tone(220, 0.12, 'sine', 0.2, 90); },
  slosh: function () { snd_noise(0.14, 'lowpass', 500, 0.18, 0.7, 0.03); },
  pour: function (d) { snd_noise(d || 1.2, 'bandpass', 700, 0.18, 0.8, (d || 1.2) * 0.4); },
  clunk: function () { snd_tone(120, 0.12, 'triangle', 0.2, 70); snd_noise(0.05, 'lowpass', 400, 0.15); },
  ratchet: function () { for (var i = 0; i < 8; i++) snd_tone(1100 + i * 40, 0.03, 'square', 0.04, null, i * 0.1); },
  ding: function () { snd_tone(1320, 1.2, 'sine', 0.18); snd_tone(2640, 0.9, 'sine', 0.08); },
  squelch: function () { snd_noise(0.18, 'lowpass', 400, 0.3, 0.7); snd_tone(260, 0.12, 'sine', 0.12, 80); },
  blup: function () { var f = 120 + Math.random() * 80; snd_tone(f, 0.09, 'sine', 0.1, f * 1.6); },
  shhk: function () { snd_noise(0.07, 'highpass', 3000, 0.15, 0.7); },
  hiss: function () { snd_noise(0.08, 'highpass', 4500, 0.05, 0.7, 0.02); },
  crunch: function () { snd_noise(0.09, 'bandpass', 1200 + Math.random() * 2200, 0.35, 1.2); },
  pat: function () { snd_noise(0.06, 'lowpass', 350, 0.25); snd_tone(150, 0.06, 'sine', 0.12, 80); },
  scratch: function () { snd_noise(0.04, 'highpass', 5000, 0.05, 0.7); },
  nom: function () { snd_tone(110, 0.1, 'sine', 0.2, 70); snd_tone(100, 0.1, 'sine', 0.2, 60, 0.2); },
  chuff: function () { snd_noise(0.09, 'bandpass', 900, 0.14, 0.8); },
  whistle: function () { snd_tone(587, 0.35, 'sine', 0.12); snd_tone(494, 0.35, 'triangle', 0.12, null, 0.35); },
  knock: function () { snd_tone(140, 0.12, 'sine', 0.35, 90); snd_noise(0.05, 'lowpass', 600, 0.15); },
  join: function () { snd_tone(440, 2.0, 'sine', 0.1, null, 0, 0.02); },
  paper: function () { snd_noise(0.25, 'highpass', 2500, 0.1, 0.7, 0.05); },
  whoosh: function () { snd_noise(0.5, 'bandpass', 600, 0.1, 0.6, 0.2); }
};
