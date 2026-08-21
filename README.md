# Clubix Compressor

Algoritmo inspirado na familia LZ desenvolvido em conjunto com plataforma web para suprir demanda interna de armazenamento.


Uso atual:
  
  Compactar/Comprimir:
  ./Clubix-compressor.exe "nome-do-arquivo"
  
  Descompactar/Descomprimir:
  ./Clubix-compressor.exe decompress "nome-do-comprimido.tzp"


### Funcionamento
Algoritmo baseado na familia LZ (substitui ocorrencias iguais por refêrencias).
Usando token LZ4-like (token: u8), 4 bits para nibble alto e 4 bits para nibble baixo, dessa forma mapeando numero de literais e tamanho do match (fora os bytes de extensão, quando necessários).
Ocorrencias encontradas são mapeadas em hash table para evitar busca O(n^2).

Representação de sequence:

Vec<u8>
[nibble alto, nibble baixo] [expansao de literais (quando nibble alto estourar) ] [ literais ] [ offset (2bytes) ] [ expansao de match (quando nibble baixo estourar )]


Como a leitura do arquivo é feita sem chunking não recomendo o uso para arquivos de tamanho elevado.



#### todo:
- multithreading;
- leitura de arquivos por chunking (leitura por offset maximo)
- matching por sobreposição para offsets muito curtos

#### Limitações
Relacionadas ao todo atual.

- Repeticoes proximas com offset pequeno tem reaproveitamento baixo ou nulo.

- Arquivos com baixa redundância (ou já comprimidos) tem um aproveitamento baixo.

- Arquivo é carregado em memória antes da compressão

- Extensão do arquivo é truncada em 4 bytes no header