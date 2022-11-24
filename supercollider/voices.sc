// Lex: This are the SynthDefs I played with

(
SynthDef(\class, {
	arg atk=2, sus=0, rel=4, c1=1, c2=(-1),
	freq=500, detune=0.2, pan=0, cfhzmin=0.1, cfhzmax=0.3, lsf=200, ldb=0,
	cfmin=500, cfmax=2000, rqmin=0.1, rqmax=0.2, amp=1, out=0;
	var sig, env;
	// env = EnvGen.kr(Env.new([0,1,1,0],[atk,sus,rel],[c1,0,c2]), doneAction:2);
	env = EnvGen.kr(Env.new([0,1,1,0],[0,1,0]), doneAction:2);
	atk.postln;
	sig = Saw.ar(freq * {LFNoise1.kr(0.5,detune).midiratio}!2);
	sig = BPF.ar(
		sig,
		{LFNoise1.kr(LFNoise1.kr(4).exprange(cfhzmin,cfhzmax)).exprange(cfmin,cfmax)}!2,
		{LFNoise1.kr(0.2).exprange(rqmin,rqmax)}!2
	);
	sig = BLowShelf.ar(sig, lsf, 0.5, ldb);
	sig = sig * env * amp;
	sig = Balance2.ar(sig[0], sig[1], pan);
	Out.ar(out, sig);
}).add;
)

(
SynthDef(\class, {
	arg atk=2, sus=0, rel=4, c1=1, c2=(-1),
	freq=500, detune=0.2, pan=0,
	cfmin=500, cfmax=2000, rqmin=0.1, rqmax=0.2, amp=1, out=0;
	var sig, env;
	env = EnvGen.kr(Env.new([0,1,1,0],[atk,sus,rel],[c1,0,c2]), doneAction:2);
	sig = Saw.ar(freq *  {LFNoise1.kr(0.5,detune).midiratio}!2);
	sig = BPF.ar(
		sig,
		{LFNoise1.kr(0.2).exprange(cfmin,cfmax)}!2,
		{LFNoise1.kr(0.1).exprange(rqmin,rqmax)}!2
	);
	sig = sig * env * amp;
	Out.ar(out, sig);
}).add;
)


(
SynthDef(\class, {
	arg freq=500, mRatio=1, cRatio=1,
	index=1, iScale=5, ic_atk=4, ic_rel=(-4),
	amp=0.2, atk=0.01, rel=3, pan=0;
	var car, mod, env, iEnv, mod2;
	iEnv = EnvGen.kr(
		Env(
			[index, index*iScale, index],
			[atk, rel],
			[ic_atk, ic_rel]
		)
	);
	env = EnvGen.kr(Env.perc(atk,rel),doneAction:2);
	mod2 = SinOsc.ar(freq/10, mul:freq/10 * iEnv);
	mod = SinOsc.ar(freq * mRatio + mod2, mul:freq * mRatio * iEnv);
	car = SinOsc.ar(freq * cRatio + mod) * env * amp;
	car = Pan2.ar(car, pan);
	Out.ar(0, car);
}).add;
)

(
~class = {
	arg size, lines, complexity;
	var freq, rel;
	// freq = ((size+20)*1.2).clip(20, 16000);
	freq = 420+(complexity*8);
	freq = freq.nearestInScale([0,7,9,10]).postln;
	// freq.postln;
	rel = 1/4+((size-1)/20);

	Synth(\class,[
		\freq, freq,
		\rel, rel,

	]);

	// (rel/4).clip(0, 1).wait;
	1.wait;
};
)

(
SynthDef(\class, {
    var snd, freq, high, lfo;
    freq = \freq.kr(440) * (Env.perc(0.001, 0.08, curve: -1).ar * 48 * \bend.kr(1)).midiratio;
    snd = Saw.ar(freq);
    snd = (snd * 100).tanh + ((snd.sign - snd) * -8.dbamp);
    high = HPF.ar(snd, 300);
    lfo = SinOsc.ar(8, [0, 0.5pi]).range(0, 0.01);
    high = high.dup(2) + (DelayC.ar(high, 0.01, lfo) * -2.dbamp);
    snd = LPF.ar(snd, 100).dup(2) + high;
    snd = RLPF.ar(snd, 7000, 2);
    snd = BPeakEQ.ar(snd, \ffreq.kr(3000) * XLine.kr(1, 0.8, 0.3), 0.5, 15);
    snd = snd * Env.asr(0.001, 1, 0.05).ar(2, \gate.kr(1));
    Out.ar(\out.kr(0), snd * \amp.kr(0.1));
}).add;
)

(
~class = {
	arg size, lines, complexity;
	var freq, rel;
	// freq = ((size+20)*1.2).clip(20, 16000);
	freq = 120+(complexity*8);
	freq = freq.nearestInScale([0,7,9,10]) /* wait here */ .postln;
	// freq.postln;
	rel = 1/4+((size-1)/20);

	Synth(\class,[
		\freq, freq,
		\rel, rel,

	]);
	// Synth(\class);
	(rel/4).clip(0, 1).wait;
};
)