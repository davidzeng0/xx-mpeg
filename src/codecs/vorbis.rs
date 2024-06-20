use super::av::*;
use super::*;

codec_pair!(CodecId::Vorbis, None, AV_CODEC_ID_VORBIS, Vorbis);
parser_pair!(CodecId::Vorbis, AV_CODEC_ID_VORBIS, Vorbis);
