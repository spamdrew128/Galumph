EXE := galumph_v0

TB := N
FEATURES :=

ifeq ($(TB),Y)
	FEATURES = --features engine/syzygy
endif

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
	V1NAME := $(EXE)-x86_64-win-v1.exe
	V2NAME := $(EXE)-x86_64-win-v2.exe
	V3NAME := $(EXE)-x86_64-win-v3.exe
	V4NAME := $(EXE)-x86_64-win-v4.exe
	TUNER_NAME := $(TUNER_EXE).exe
	GENERATOR_NAME := $(GENERATOR_EXE).exe

	RM_COMMAND := del
else
	NAME := $(EXE)
	V1NAME := $(EXE)-x86_64-linux-v1
	V2NAME := $(EXE)-x86_64-linux-v2
	V3NAME := $(EXE)-x86_64-linux-v3
	V4NAME := $(EXE)-x86_64-linux-v4
	TUNER_NAME := $(TUNER_EXE)
	GENERATOR_NAME := $(GENERATOR_EXE)

	RM_COMMAND := rm
endif

rule:
	cargo rustc --release $(FEATURES) -- -C target-cpu=native --emit link=$(NAME)

release:
	cargo rustc --release $(FEATURES) -- -C target-cpu=x86-64 --emit link=$(V1NAME)
	cargo rustc --release $(FEATURES) -- -C target-cpu=x86-64-v2 --emit link=$(V2NAME)
	cargo rustc --release $(FEATURES) -- -C target-cpu=x86-64-v3 --emit link=$(V3NAME)
	cargo rustc --release $(FEATURES) -- -C target-cpu=x86-64-v4 --emit link=$(V4NAME)

bench:
	cargo r --release bench

clean:
	$(RM_COMMAND) $(NAME) $(V1NAME) $(V2NAME) $(V3NAME) $(V4NAME)
	