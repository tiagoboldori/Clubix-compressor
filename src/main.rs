

// Estrutura definida para o frame: [header][dados]
// header:  [TAMANHO DO ARQUIVO, EXTENSÃO ORIGINAL]
// corpo : [dados]

//dados -> logica do LZ4: sequencias de [sequence]
// cada sequence: [token][literals][offset] ou [token][literals]
// cada token tem 1 byte [nibble alto, nibble baixo]
// nibble alto: quantidade de literals 
// nibble baixo: tamanho do match - 4

// minmatch de 4 bytes/caracteres

const MINMATCH: usize = 4;

fn main() {
    
    let texto:String = String::from("Olá, computador, computador. Este é um texto de teste para testar o compressor manual do Clubix!. Qualquer semelhança com outro compressor é mera coincidência. computadores são legais!");
    let dados: &[u8] = texto.as_bytes();

    let mut saida:Vec<u8> = Vec::new();

    let mut p:usize = 4;

    let mut aux:usize = 0;
    
    while p<dados.len(){
        let aux_char = dados[p] as char;
        println!("{}", aux_char);
        p= p + 1 ;
    }
    
    
    println!("Hello, world!");
}
