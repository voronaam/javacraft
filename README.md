JVM byte code parser for FreeMiner/Minetest and Music
===
This is a playground project that seeks to extract some very basic information about java program and write a Freeminer/Minetest map out of it.

This can be used as a way to bring to live and interactive visualization of any JVM (Java, Scala, etc) code.

Current version outputs an SQL script that could be executed against an existing sqlite3 database defining the map. Leveldb is not supported (yet).

MELO mode
===

With `--melo` parameter it will generate a [MELO](https://github.com/mistodon/melo) sheet of music based on the code. Not sure why you would need it, but it was fun to write.

SuperCollider mode
===

With `--super` parameter the program will output a snippet of SuperCollider code that can be used to generate an audio representation
of a JAR or class file.

You still need to come up with a "voice" for it. For example, I have been playing with this routine.
This routine is based on the snippets from [Eli Fieldsteel SuperCollider tutorial series](https://www.youtube.com/@elifieldsteel/).
All the creative credit belong to this awesome musician and teacher.

```
(
SynthDef(\class, {
	arg atk=2, sus=0, rel=4, c1=1, c2=(-1),
	freq=500, detune=0.2, pan=0, cfhzmin=0.1, cfhzmax=0.3, lsf=200, ldb=0,
	cfmin=500, cfmax=2000, rqmin=0.1, rqmax=0.2, amp=1, out=0;
	var sig, env;
	env = EnvGen.kr(Env.new([0,1,1,0],[atk,sus,rel],[c1,0,c2]), doneAction:2);
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
~class = {
	arg size, lines, complexity;
	Synth(\class, [
			\freq, 10*complexity+10,
		    \atk, 1+(size/5),
		    \rel, 1+(lines/5),
			\amp, 0.5,
			\cfmin, 50*2,
			\cfmax, 50*50,
			\rqmin, 0.01,
			\rqmax, 0.05,
		]);
	1.wait;
};
)

(
~r = Routine.new({

PASTE THE SNIPPET HERE
   
	6.wait;
	CmdPeriod.run;
});
)

~r.reset.play;
```

Dependencies
===

Rust (https://www.rust-lang.org/learn/get-started)
`sqlite` (libsqlite3-dev on Debian)

Sample usage:
===
  cargo run path/to/soure.class path/to/other.jar

    This will print out static analysis results to stdout but will not attempt to generate a map.

  cargo run -- --map=/path/to/map.sqlite path/to/soure.class path/to/other.jar

    This will print out static analysis results to stdout and will create a codecity in the supplied map.
