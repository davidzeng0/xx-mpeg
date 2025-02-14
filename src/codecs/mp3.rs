use super::av::*;
use super::*;

codec_pair!(CodecId::Mp3, None, AV_CODEC_ID_MP3, Mp3);
parser_pair!(CodecId::Mp3, AV_CODEC_ID_MP3, Mp3);
