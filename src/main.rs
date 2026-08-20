
// Estrutura definida para o frame: [header][dados]
// header:  [TAMANHO DO ARQUIVO, EXTENSÃO ORIGINAL]
// corpo : [dados]

//dados -> logica do LZ4: sequencias de [sequence]
// cada sequence: [token][literals][offset] ou [token][literals]
// cada token tem 1 byte [nibble alto, nibble baixo]
// nibble alto: quantidade de literals 
// nibble baixo: tamanho do match - 4

// minmatch de 4 bytes/caracteres


//TO DO:
// Trocar estratégia de matching. Atual: Primeiro match. Ideal: Match mais longo.
// Aplicar correções (essencial antes de prosseguir para manipulacao de arquivos.)
// Implementar e adotar o uso de arquivos
// refatoração e reestruturação / organização do codigo:
//          - colocar compressao em funcao propria
//          - organizar 

//CORREÇÕES:
// - Flush ao final do loop de compressao (feito parcial, necessario testes)
// - Extensao para match_len > 15 (extensão para nibble baixop)

use std::env;

const G_RATIO: f64 = 1.618;

//2^32 / g_ratio
const FIB_HASH_MULT:u32 = 2654435761;

const MINMATCH: usize = 4;
const MAX_OFFSET: usize = 65535;



// funcao simples para englobar decompress()
fn decompress_file(file_name:&String){
    println!("Iniciando descompressao | Hora: {:?}", std::time::SystemTime::now());
    let arquivo = std::fs::read(file_name);

    let dados_dec = match arquivo{
        Ok(dados) => dados,
        Err(err) =>{
            println!("Erro");
            return;
        }
    };


    let extensao = String::from_utf8_lossy(
        &dados_dec[0..4].iter().cloned().take_while(|&b| b != 0).collect::<Vec<u8>>()
    ).into_owned();

    let saida = decompressor(&dados_dec);

    let path = std::path::Path::new(file_name);
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let nome_saida = parent.join(format!("{}_.{}", stem, extensao));

    match std::fs::write(&nome_saida, saida){
        Ok(_) => println!("Arquivo salvo em {}", nome_saida.display()),
        Err(err) => println!("Erro ao salvar arquivo: {}", err),
    }
}



fn decompressor(dados_dec: &Vec<u8> ) -> Vec<u8>{

    let mut saida: Vec<u8> = Vec::new();
    
    let mut p:usize = 4;
    
    while p<dados_dec.len(){

        let token_pos: usize = p;
        let token = dados_dec[token_pos];

        let mut literal_count: u128 = ((token >>4) &0x0F )as u128;
        
        let match_len: usize = (token &0x0F) as usize;
        
        let mut offset: usize;
        
        let mut ext_bytes:usize = 0;
        
        if literal_count >=15{

            ext_bytes+=1;

            while ((dados_dec[p+ext_bytes]) as u16) >= 255{
               literal_count += dados_dec[p+ext_bytes] as u128; 
               ext_bytes+= 1;
            }
            
            if (dados_dec[p+ext_bytes] as u16) < 255 {
                literal_count += dados_dec[p+ext_bytes] as u128;
            }
            
        }

        if dados_dec.len() > p + ext_bytes + 2 + literal_count as usize{
            let l: u16 = dados_dec[p + ext_bytes + 1 + literal_count as usize] as u16;
            let h: u16 = dados_dec[p + ext_bytes + 2 + literal_count as usize] as u16;
            offset = (l | (h << 8)) as usize;
        }else{
            offset = 0;
        }
        

        saida.extend_from_slice(&dados_dec[p + ext_bytes + 1.. p + ext_bytes + (literal_count as usize) + 1]);
        
        // logica para encontrar slice a ser copiado:
        // p + literal = fim do slice, ou seja, basta subtrair o offset disso para saber onde inicia o match
        // a partir da posicao do match (p+literais-offset) basta somar o match_len + MINMATCH para saber o tamanho do match, temos:
        // dados[p + literal - offset .. p + literal - offset + match_len + MINMATCH]
        

        if offset > 0{
            let start = saida.len() - offset;
            for i in 0..(match_len + MINMATCH){
                let byte  = saida [start+i];
                saida.push(byte);
            }
        }
        

        //println!("Saida (descomprimida): {}", String::from_utf8_lossy(&saida));
        
        //verificar p (posição no comeco da sequencia de literais)
        
        p = p + literal_count as usize + ext_bytes + 3;


    }

    //println!("Saida (descomprimida): {}", String::from_utf8_lossy(&saida));
    println!("Descompressao finalizada | Hora: {:?}", std::time::SystemTime::now());

    saida
}




// funcao para pegar extensao de arquivos (salva no header para uso durante a descompressao)
fn read_file_extension(file_name: &String) -> [u8;4]{
    let extensao = std::path::Path::new(file_name)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let mut header: [u8; 4] = [0; 4];
    for (i, b) in extensao.bytes().take(4).enumerate() {
        header[i] = b;
    }
    header
}




//hashing
fn fib_hashing(bytes: &[u8], p:usize, hash_bits: u32) -> usize { 
    let seq: u32 = u32::from_le_bytes([bytes[p], bytes[p+1], bytes[p+2], bytes[p+3]]);
    
    let h: u32 = seq.wrapping_mul(FIB_HASH_MULT);
    (h >> (32 - hash_bits)) as usize
}



//              COMPRESSOR
fn main() {
    
    //let texto:String = String::from("Ola, computador, computador. Este e um texto de teste para testar o compressor manual do Clubix!. Qualquer semelhanca com outro compressor e mera coincidencia. computadores sao legais! Um compressor serve, principalmente, para comprimir arquivos. A ideia e que eles gastem a menor quantidade de espaco possivel no disco do computador.Ola Mundo.");
    //let dados: &[u8] = texto.as_bytes();

    
    let args: Vec<String> = std::env::args().collect();

    if args[1] == "decompress"{
        decompress_file(&args[2]);
        std::process::exit(0);
    }

    let file_name : &String = &args[1];
    

    let header = read_file_extension(file_name);    

    let arquivo =std::fs::read(&args[1]);
    let dados = match arquivo{
        Ok(T) => T,
        Err(err) =>{
            println!("Erro ao ler bytes do arquivo");
            return ();
        }
    };


    let mut saida:Vec<u8> = Vec::new();
    saida.extend_from_slice(&header);


    let hash_bits:u32 = 16;
    let table_size: usize = 1 << hash_bits;
    let mut table: Vec<Option<usize>> = vec![None;table_size];


    let mut p:usize = 0;
    let mut p_end:usize = p+MINMATCH;

    let mut literal_count:u128 = 0;
    
    let mut token_pos: usize = 0;
    
    println!("Iniciando Compressão | Hora: {:?} ", std::time::SystemTime::now());
    
    while p_end<dados.len(){
        
        //println!(" Byte atual[{}..{}] -> {} ", p, p_end, String::from_utf8_lossy(&dados[p..p_end]));
        let idx_hash: usize = fib_hashing(&dados, p, hash_bits);

        
        let mut count: usize  = p;
        let mut b_match:bool = false;

        let mut count_end:usize = count + MINMATCH;

        match table[idx_hash]{
            Some(c) =>{
                count = c;
                count_end = count + MINMATCH;
                while true {
                    if p_end-p<=15 && dados[count..count_end] == dados[p..p_end] && count_end<=p && p_end < dados.len() && p - c <= MAX_OFFSET{

                        b_match = true;

                        count_end += 1;
                        p_end += 1;               
                        
                    }else{
                        count_end -= 1;
                        p_end = p + MINMATCH;
                        break;
                    }
                }
            }
            None=>{ 
                ()
            }
        }
        
        table[idx_hash] = Some(p);


        if b_match==true {
            
            // transferir count de literais para nible alto
            // criar nible baixo com o  match len
            //adicionar / alterar token
            
            //println!("{}",literal_count);

            let m_size: usize = count_end - count;
            let match_len:u8 = (m_size - MINMATCH) as u8;
            
            let offset: u16= (p-count) as u16;

            let mut token:u8; 

            if literal_count>=15{

                token = ((15& 0x0F) << 4) | (match_len& 0x0F);

                saida.push(token);

                literal_count-=15; 
                
                while literal_count>=255{
                    saida.push(255);
                    literal_count= literal_count.saturating_sub(255);
                }
                saida.push(literal_count as u8);

            }else{

                token = (((literal_count) as u8 & 0x0F) << 4) | (match_len& 0x0F);
                saida.push(token);

            }


            saida.extend_from_slice(&dados[token_pos..p]);
            saida.extend_from_slice(&offset.to_le_bytes());


            literal_count = 0;
            p +=  (match_len) as usize + (MINMATCH) as usize;
            p_end = p + MINMATCH;
            token_pos = p;

            //println!("token -> literals:{} | Tamanho do match:{} | Offset: {}",(token >> 4) & 0x0F, token & 0x0F, offset);

        }else{
            
            p = p + 1;
            p_end = p + MINMATCH;
            literal_count+=1;
        }
        
    }
    
    // flush final apos loop para garantir saida de todos os bytes do arquivo
    // criar token com nibble baixo = 0

    let mut token:u8;

    if token_pos<dados.len(){
        println!("Faltou flush...");
        println!("Flushing...");
        println!("Posição do ponteiro token_pos {}", token_pos);
        
        literal_count = (dados[token_pos ..].len()) as u128;

        if literal_count  >= 15{
            println!("Explodiu");
            token = ((15& 0x0F) << 4) | (0 & 0x0F);
            saida.push(token);
            literal_count-=15;

            while literal_count>=255{
                saida.push(255);
                literal_count= literal_count.saturating_sub(255);
            }
            saida.push(literal_count as u8);
            
            saida.extend_from_slice(&dados[token_pos .. ]);


        }else{
            token = ((literal_count as u8 & 0x0F) << 4) | (0 & 0x0F);
            saida.push(token);
            saida.extend_from_slice(&dados[token_pos .. ]);
        }

    }
    
    println!("Tamanho final da saída {} | entrada {}  | Taxa de compressao: {} ", saida.len(), dados.len(), (1 as f32 - (saida.len()) as f32/(dados.len()) as f32) );
    //println!("Saida (bytes): {:?}", saida);
    //println!("Saida (lossy string): {}", String::from_utf8_lossy(&saida));

    //salvando arquivo
    let path = std::path::Path::new(file_name);
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let nome_saida = parent.join(format!("{}.tzp", stem));

    match std::fs::write(&nome_saida, &saida){
        Ok(_) => println!("Arquivo salvo em {}", nome_saida.display()),
        Err(err) => println!("Erro ao salvar arquivo: {}", err),
    }
    println!("Compressão finalizada | Hora: {:?} ", std::time::SystemTime::now());

    //decompressor(&saida);

}
