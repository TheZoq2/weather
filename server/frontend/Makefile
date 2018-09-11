all:
	make js
	./build_index.hs > output/index.html

js:
	elm-make src/Main.elm --output output/index.js
