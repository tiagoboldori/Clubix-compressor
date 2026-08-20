Algoritmo inspirado na familia LZ desenvolvido em conjunto com plataforma web para suprir demanda interna de armazenamento.


Uso atual:
  
  Compactar/Comprimir:
  ./Clubix-compressor.exe "nome-do-arquivo"
  
  Descompactar/Descomprimir:
  ./Clubix-compressor.exe decompress "nome-do-comprimido.txp"


todo:
- refatoração de código;
- expansão de nibble (matching);
- multithreading;
- leitura de arquivos por chunking (leitura por offset maximo)

