use super::av::*;
use super::*;

codec_pair!(CodecId::Flac, None, AV_CODEC_ID_FLAC, Flac);
parser_pair!(CodecId::Flac, AV_CODEC_ID_FLAC, Flac);
